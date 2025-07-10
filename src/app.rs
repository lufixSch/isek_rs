use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use chrono::Utc;
use color_eyre::eyre::Result;
use colors_transform::Rgb;
use eyre::{Context, ContextCompat, eyre};
use ical::{
    IcalParser, generator::Emitter, parser::ical::component::IcalCalendar, property::Property,
};
use icalendar::{Calendar, CalendarComponent, Component, Todo, TodoStatus};
use ratatui::widgets::ListState;

use crate::{
    config::{
        CalendarConfig, DisplayOptions, FilterConfig, IsekConfig, ShowDoneOptions, SortingConfig,
        SortingVariant,
    },
    helper::{ICAL_UTC_DATE_TIME_FORMAT, calculate_index, ical_datetime_to_chrono},
};

#[derive(Debug)]
pub enum State {
    Normal,
    Interactive,
    ConfigSort,
    ConfigFilter,
}

#[derive(Debug)]
pub enum CalData {
    // Saves IcalCalendar and iCalendar representation of the file mapped to its file name
    // NOTE: IcalCalendar is used to modify (more consistent behaviour) while iCalendar is used for representation
    VDIR(HashMap<String, (IcalCalendar, Calendar)>),
}

/// Representation of a ToDo item
#[derive(Debug)]
pub struct IsekTodo<'a> {
    pub calendar_name: &'a String,
    pub color: &'a Rgb,
    data: &'a Todo,
}

impl<'a> IsekTodo<'a> {
    pub fn get(&self) -> &Todo {
        self.data
    }

    // pub fn get_mut(&mut self) -> &mut Todo {
    //     self.data
    // }
    //
    // pub fn toggle_done(&mut self) -> bool {
    //     match self.data.get_completed() {
    //         Some(_) => {
    //             let old_properties = self.data.properties().clone();
    //
    //             *self.data = Todo::new();
    //
    //             for (key, property) in old_properties {
    //                 if ["COMPLETED", "STATUS", "PERCENT-COMPLETE"].contains(&key.as_str()) {
    //                     continue;
    //                 }
    //
    //                 self.data.append_property(property);
    //             }
    //
    //             false
    //         }
    //         None => {
    //             self.data.completed(Utc::now());
    //             self.data.percent_complete(100);
    //             self.data.status(TodoStatus::Completed);
    //             true
    //         }
    //     }
    //
    //     // TODO: self.data.last_modified(Utc::now());
    //     // res
    // }
}

/// Representation of a calendar with its configuration and data
#[derive(Debug)]
pub struct IsekCalendar {
    /// Configuration for this calendar
    pub config: CalendarConfig,
    /// Display name of the calendar
    pub name: String,
    /// Color associated with the calendar (used for display)
    pub color: Rgb,
    /// Parsed iCalendar data containing todos and events
    data: CalData,
}

impl IsekCalendar {
    /// Create a new calendar instance from configuration
    pub fn from_config(cfg: CalendarConfig) -> Result<Self> {
        match cfg {
            CalendarConfig::VDIR(ref config) => {
                // Read all .ics files in directory
                let dir_path = Path::new(&config.path);
                let entries = fs::read_dir(dir_path)?;
                let mut cal: HashMap<String, (IcalCalendar, Calendar)> = HashMap::new();

                // Get calendar display name from file
                let name_path = dir_path.join("displayname");
                let name = fs::read_to_string(name_path)?;

                // Get and parse color configuration
                let color_path = dir_path.join("color");
                let color_string = fs::read_to_string(color_path)?;
                let color = Rgb::from_hex_str(&color_string).map_err(|err| {
                    eyre!(
                        "Unable to read color for calendar '{}': {}",
                        name,
                        err.message
                    )
                })?;

                // Process all .ics files in the directory
                for entry in entries.flatten() {
                    let path = entry.path();

                    // Check if it is a file and has .ics extension
                    if path.is_file()
                        && path
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("ics"))
                    {
                        match File::open(&path) {
                            Ok(f) => {
                                let buf = BufReader::new(f);
                                let ical = IcalParser::new(buf).last().and_then(|res| res.ok());

                                match ical {
                                    Some(ical) => {
                                        if ical.todos.is_empty() {
                                            // Skip files without Todos
                                            continue;
                                        }

                                        match fs::read_to_string(&path) {
                                            Ok(content) => {
                                                match content.parse() {
                                                    Ok(parsed_calendar) => {
                                                        cal.insert(
                                                            String::from(
                                                                path.file_name()
                                                                    .unwrap()
                                                                    .to_str()
                                                                    .unwrap(),
                                                            )
                                                            .trim_end_matches(".ics")
                                                            .to_owned(), // WARN: File name should always exist and be valid
                                                            (ical, parsed_calendar),
                                                        );
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Error parsing file {}: {}",
                                                            path.display(),
                                                            e
                                                        )
                                                    }
                                                }
                                            }
                                            Err(e) => eprintln!(
                                                "Error reading file {}: {}",
                                                path.display(),
                                                e
                                            ),
                                        }
                                    }
                                    None => {
                                        eprintln!(
                                            "IcalParser: Error parsing file {}",
                                            path.display(),
                                        )
                                    }
                                }
                            }
                            Err(e) => eprintln!("Error reading file {}: {}", path.display(), e),
                        }
                    }
                }

                Ok(Self {
                    name,
                    color,
                    config: cfg,
                    data: CalData::VDIR(cal),
                })
            }
        }
    }

    // Save changes to file
    pub fn save(&mut self) -> Result<()> {
        match self.config {
            CalendarConfig::VDIR(ref config) => {
                let dir_path = Path::new(&config.path);

                if !dir_path.exists() {
                    return Err(eyre!("Calendar path doesn't exist"));
                };

                match &mut self.data {
                    CalData::VDIR(cal) => {
                        // Save each calendar as an .ics file
                        for (id, (ical, calendar)) in cal {
                            let filename = format!("{id}.ics");
                            let ics_path: PathBuf = dir_path.join(filename);

                            let content = ical.generate();
                            fs::write(&ics_path, content.clone()).with_context(|| {
                                format!("Failed to write iCalendar file at {}", ics_path.display())
                            })?;

                            *calendar = content.parse().map_err(|e| {
                                eyre!("Could not update iCalendar from ical representation: {}", e)
                            })?
                        }

                        Ok(())
                    }
                }
            }
        }
    }

    pub fn get_todos(&self) -> Vec<IsekTodo> {
        match &self.data {
            CalData::VDIR(cals) => cals
                .iter()
                .flat_map(|(_, (_, cal))| {
                    cal.components
                        .iter()
                        .flat_map(|c| {
                            if let CalendarComponent::Todo(t) = c {
                                Some(IsekTodo {
                                    calendar_name: &self.name,
                                    color: &self.color,
                                    data: t,
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<IsekTodo>>()
                })
                .collect(),
        }
    }

    pub fn get_todo(&self, id: &str) -> Option<&Todo> {
        match &self.data {
            CalData::VDIR(cals) => cals.get(id).and_then(|(_, cals)| {
                cals.components.first().and_then(|c| {
                    if let CalendarComponent::Todo(t) = c {
                        Some(t)
                    } else {
                        None
                    }
                })
            }),
        }
    }

    pub fn toggle_done(&mut self, id: &str) -> Option<()> {
        match &mut self.data {
            CalData::VDIR(cals) => {
                let (ical, cal) = cals.get_mut(id)?;
                let todo = ical.todos.get_mut(0)?;

                let completed_idx = todo.properties.iter().position(|p| p.name == "COMPLETED");
                match completed_idx {
                    // Task is marked complete => undo
                    Some(idx) => {
                        todo.properties.swap_remove(idx);

                        if let Some(idx) = todo.properties.iter().position(|p| p.name == "STATUS") {
                            todo.properties.swap_remove(idx);
                        }

                        if let Some(idx) = todo
                            .properties
                            .iter()
                            .position(|p| p.name == "PERCENT-COMPLETE")
                        {
                            todo.properties.swap_remove(idx);
                        }
                    }

                    // Task is NOT marked complete => mark complete
                    None => {
                        todo.properties.push(Property {
                            name: String::from("COMPLETED"),
                            params: None,
                            value: Some(Utc::now().format(ICAL_UTC_DATE_TIME_FORMAT).to_string()),
                        });

                        match todo.properties.iter().position(|p| p.name == "STATUS") {
                            Some(idx) => {
                                todo.properties[idx].value = Some(String::from("COMPLETED"))
                            }
                            None => {
                                todo.properties.push(Property {
                                    name: String::from("STATUS"),
                                    params: None,
                                    value: Some(String::from("COMPLETED")),
                                });
                            }
                        }

                        match todo
                            .properties
                            .iter()
                            .position(|p| p.name == "PERCENT-COMPLETE")
                        {
                            Some(idx) => todo.properties[idx].value = Some(100.to_string()),
                            None => {
                                todo.properties.push(Property {
                                    name: String::from("PERCENT-COMPLETE"),
                                    params: None,
                                    value: Some(100.to_string()),
                                });
                            }
                        }
                    }
                }

                // Update iCalendar Representation
                *cal = ical.generate().parse().ok()?;

                Some(())
            }
        }
    }
}

/// Representation of all calendars
#[derive(Debug)]
pub struct IsekCalendars {
    data: HashMap<String, IsekCalendar>,
}

impl IsekCalendars {
    fn from_config(cfg: Vec<CalendarConfig>) -> Result<Self> {
        Ok(Self {
            data: cfg
                .into_iter()
                .map(IsekCalendar::from_config)
                .map(|cal| cal.map(|cal| (cal.name.clone(), cal)))
                .collect::<Result<HashMap<String, IsekCalendar>>>()?,
        })
    }

    pub fn get_todos(
        &self,
        sort: Option<&SortingConfig>,
        filter: Option<&FilterConfig>,
    ) -> Vec<IsekTodo> {
        let mut todos: Vec<IsekTodo> = self
            .data
            .iter()
            .flat_map(|(_, cal)| cal.get_todos())
            .collect();

        if let Some(filter) = filter {
            match filter.show_done {
                ShowDoneOptions::Hide => {
                    todos.retain(|t| t.get().get_completed().is_none());
                }
                ShowDoneOptions::Some => {
                    todos.retain(|t| {
                        let completed = t.get().get_completed();

                        if let Some(dt) = completed {
                            let diff = Utc::now() - dt;

                            diff.num_days() < filter.show_done_for as i64
                        } else {
                            true
                        }
                    });
                }
                ShowDoneOptions::Show => {}
            }
        }

        if let Some(sort) = sort {
            match sort.by {
                SortingVariant::Date => {
                    todos.sort_by(|a, b| {
                        let a_due = a.get().get_due();
                        let b_due = b.get().get_due();

                        if a.get().get_completed().is_some() {
                            if b.get().get_completed().is_some() {
                                return Ordering::Equal;
                            };

                            return Ordering::Greater;
                        }

                        if b.get().get_completed().is_some() {
                            return Ordering::Less;
                        }

                        if let Some(a_due) = a_due {
                            if let Some(b_due) = b_due {
                                return a_due.date_naive().cmp(&b_due.date_naive());
                            };

                            return Ordering::Less;
                        };

                        if b_due.is_none() {
                            return Ordering::Greater;
                        };

                        Ordering::Equal
                    });
                }
                SortingVariant::Priority => {
                    todos.sort_by(|a, b| {
                        let a_prio = a.get().get_priority().unwrap_or(10);
                        let b_prio = b.get().get_priority().unwrap_or(10);

                        if a.get().get_completed().is_some() {
                            if b.get().get_completed().is_some() {
                                return Ordering::Equal;
                            };

                            return Ordering::Greater;
                        }

                        if b.get().get_completed().is_some() {
                            return Ordering::Less;
                        }

                        a_prio.cmp(&b_prio)
                    });
                }
                SortingVariant::Index => {
                    let now = Utc::now();

                    todos.sort_by(|a, b| {
                        let a_dt = a
                            .get()
                            .get_due()
                            .map(ical_datetime_to_chrono)
                            .unwrap_or(Utc::now());
                        let a_prio = a.get().get_priority().unwrap_or(10);

                        let b_dt = b
                            .get()
                            .get_due()
                            .map(ical_datetime_to_chrono)
                            .unwrap_or(Utc::now());
                        let b_prio = b.get().get_priority().unwrap_or(10);

                        calculate_index(&a_prio, &a_dt, &now)
                            .total_cmp(&calculate_index(&b_prio, &b_dt, &now))
                    });
                }
            };

            if !sort.ascending {
                todos.reverse();
            }
        };

        todos
    }

    pub fn get_todo(&self, calendar_id: &str, id: &str) -> Option<&Todo> {
        if let Some(cal) = self.data.get(calendar_id) {
            cal.get_todo(id)
        } else {
            None
        }
    }

    pub fn toggle_done(&mut self, calendar_id: &str, id: &str) -> Option<()> {
        if let Some(cal) = self.data.get_mut(calendar_id) {
            cal.toggle_done(id)
        } else {
            None
        }
    }
}

/// Main application state and logic
#[derive(Debug)]
pub struct App {
    /// Flag to indicate if the application should exit
    pub exit: bool,

    /// Indicates the state, the application is in
    pub state: State,

    /// All calendars in isek
    pub calendars: IsekCalendars,

    /// Settings defining how the data is displayed
    pub display: DisplayOptions,

    /// State for list navigation (selection)
    pub list_state: ListState,
}

impl App {
    /// Create and initialize a new application instance
    pub fn new() -> Result<Self> {
        // Load configuration from file
        let config: IsekConfig = confy::load("isek", "config")?;

        Ok(Self {
            exit: false,
            state: State::Normal,
            calendars: IsekCalendars::from_config(config.calendars)?,
            display: config.display,
            list_state: ListState::default(),
        })
    }

    /// Switch to another application state
    pub fn switch_state(&mut self, state: State) {
        self.state = state
    }

    /// Set the exit flag to true, which will terminate the application loop
    pub fn exit(&mut self) {
        self.exit = true
    }

    /// Clear list selection (used for escape key)
    pub fn escape(&mut self) {
        self.list_state.select(None);
        self.state = State::Normal
    }

    /// Change sorting config (as long as the program runs) and switch back to normal mode
    pub fn configure_sort(&mut self, sort: SortingConfig) -> Result<()> {
        self.display.sort = sort;
        self.escape();

        Ok(())
    }

    /// Change filter config (as long as the program runs) and switch back to normal mode
    pub fn configure_filter(&mut self, filter: FilterConfig) -> Result<()> {
        self.display.filter = filter;
        self.escape();

        Ok(())
    }

    /// Mark currently selected task as done
    /// If already completed mark as uncompleted
    pub fn toggle_done(&mut self) -> Result<()> {
        match self.list_state.selected() {
            Some(task_idx) => {
                let tasks = self
                    .calendars
                    .get_todos(Some(&self.display.sort), Some(&self.display.filter));

                match tasks.get(task_idx) {
                    Some(task) => {
                        let cal_id = task.calendar_name.clone();
                        let task_id = task.get().get_uid().wrap_err("Task has no UID")?.to_owned();

                        self.calendars.toggle_done(&cal_id, &task_id);

                        let cal = self.calendars.data.get_mut(&cal_id).wrap_err(format!(
                            "Unable to save changes! Could not find calendar '{}'.",
                            cal_id
                        ))?;
                        cal.save()?;

                        Ok(())
                    }
                    None => Ok(()),
                }
            }
            None => Ok(()),
        }
    }
}
