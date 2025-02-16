use iced::widget::{button, checkbox, column, container, focus_next, row, text, text_input, Row};
use iced::{Element, Length, Right, Subscription, Task, time};
use std::time::Duration;
use std::vec;
use uuid::Uuid;
use taskmanager::project::{self, ProjectData, Project};

#[derive(Debug, Clone)]
enum Context {
	ProjectList,
	NewProject,
	EditProject,
}

#[derive(Debug, Clone)]
struct ProjectListState {
	selected_projects: Vec<Uuid>,
}

impl ProjectListState {
	fn new() -> Self {
		Self {
			selected_projects: Vec::new(),
		}
	}
}

#[derive(Debug, Clone)]
struct FormFieldState {
	is_valid: bool,
	validation_message: String,
}

impl FormFieldState {
	fn new() -> Self {
		Self {
			is_valid: true,
			validation_message: "".to_string(),
		}
	}
}

#[derive(Debug, Clone)]
struct ProjectFormState {
	name_field: FormFieldState,
	description_field: FormFieldState,
}

impl ProjectFormState {
	fn new() -> Self {
		Self {
			name_field: FormFieldState::new(),
			description_field: FormFieldState::new(),
		}
	}
}

#[derive(Debug, Clone)]
struct AppState {
	context: Option<Context>,
	projects_data: Option<ProjectData>,
	current_project: Option<Project>,
	project_list_state: Option<ProjectListState>,
	project_form_state: Option<ProjectFormState>,
}

#[derive(Debug)]
enum App {
	Loading,
	Loaded(AppState),
}

#[derive(Debug, Clone)]
enum Message {
	AppLoaded(Result<AppState, String>),
	AppSync,
	NewProject,
	EditProject(Uuid),
	EditProjectCancel,
	CurrentProjectSave,
	CurrentProjectNameChange(String),
	CurrentProjectDescriptionChange(String),
	ProjectListProjectSelected { selected: bool, selected_project_id: Uuid },
	ProjectListSelectAllProjects(bool),
}

impl Default for AppState {
	fn default() -> Self {
		Self {
			context: None,
			projects_data: None,
			current_project: None,
			project_list_state: None,
			project_form_state: None,
		}
	}
}

fn heading(heading_text: &str) -> Element<Message> {
	container(
		text(heading_text)
		.size(30)
	)
	.into()
}

fn form_field<'a>(label: &'a str, input: Element<'a, Message>, field: &'a FormFieldState) -> Element<'a, Message> {
	column![
		text(label),
		input,
		text(if field.is_valid { "" } else { field.validation_message.as_str() }),
	]
	.spacing(10)
	.into()
}

fn project_form<'a>(project: &'a project::Project, form: &'a ProjectFormState) -> Element<'a, Message> {
	column![
		form_field(
			"Name",
			text_input("Project Name", project.name.as_str())
				.on_input(Message::CurrentProjectNameChange)
				.into(),
			&form.name_field
		),
		form_field(
			"Description",
			text_input("Project Description", project.description.as_str())
				.on_input(Message::CurrentProjectDescriptionChange)
				.into(),
			&form.description_field
		),
	]
	.spacing(20)
	.into()
}

impl AppState {
	async fn load() -> Result<AppState, String> {
		println!("Loading data");
		let load_result = project::load_data();

		match load_result {
			Ok(projects_data) => {
				println!("Data loaded successfully");
				Ok(Self {
					context: Some(Context::ProjectList),
					projects_data: Some(projects_data),
					current_project: None,
					project_list_state: Some(ProjectListState::new()),
					project_form_state: Some(ProjectFormState::new()),
				})
			},
			Err(error) => {
				println!("Error occurred while loading data: {}", error);

				Ok(Self {
					..AppState::default()
				})
			}
		}
	}

	fn save(&self) -> Result<(), String> {
		println!("Saving data");
		let save_result = project::write_data(self.projects_data.as_ref().unwrap());

		match save_result {
			Ok(_) => {
				println!("Data saved successfully");
				Ok(())
			},
			Err(error) => {
				println!("Error occurred while saving data: {}", error);
				Ok(())
			}
		}
	}

	fn view(&self) -> Element<Message> {
		match &self.context {
			None => {
				return text("No context found").into();
			},
			Some(context) => {
				match context {
					Context::ProjectList => {
						self.view_project_list()
					},
					Context::NewProject => {
						self.view_edit_project()
					},
					Context::EditProject => {
						self.view_edit_project()
					}
				}
			}
		}
	}

	fn get_projects(&self) -> Vec<Project> {
		match &self.projects_data {
			None => Vec::new(),
			Some(projects_data) => projects_data.get_projects(),
		}
	}

	fn save_current_project(&mut self) {
		let current_project = &mut self.current_project;
		if let Some(current_project) = current_project {
			let projects_data = &mut self.projects_data;
			if let Some(projects_data) = projects_data {
				let context = &mut self.context;
				if let Some(context) = context {
					match context {
						Context::NewProject => {
							projects_data.create_project(&current_project.name, &current_project.description);
						},
						Context::EditProject => {
							let project: Option<&mut Project> = projects_data.get_project_mut(&current_project.id);
							if let Some(project) = project {
								project.name = current_project.name.clone();
								project.description = current_project.description.clone();
							}
						},
						_ => {},
					}
				}
			}
		}
	}

	fn is_project_selected(&self, project_id: &Uuid) -> bool {
		match &self.project_list_state {
			None => false,
			Some(project_list_state) => project_list_state.selected_projects.contains(project_id),
		}
	}

	fn get_selected_project_ids(&self) -> Vec<Uuid> {
		self.get_projects()
			.iter()
			.map(|project| {
				project.id
			})
			.filter(|project_id| {
				self.is_project_selected(project_id)
			})
			.collect()
	}

	fn project_list(&self) -> Element<Message> {
		let projects = &self.get_projects();
		let project_list: Vec<Element<Message>> = projects.iter().map(|project| {
			let is_project_selected = self.is_project_selected(&project.id);

			let select_project = {
				let project_id = project.id;
				move |selected: bool| {
					Message::ProjectListProjectSelected { selected, selected_project_id: project_id}
				}
			};

			Row::new()
				.push(checkbox("", is_project_selected).on_toggle(select_project))
				.push(text(project.name.clone()))
				.push(text(project.description.clone()))
				.push(button("Edit").on_press(Message::EditProject(project.id)))
				.spacing(15)
				.into()
		}).collect();

		if projects.is_empty() {
			return text("You have no projects").into();
		}

		let all_projects_selected = self.get_selected_project_ids().len() == projects.len();

		container(
			column![
				checkbox("Select All", all_projects_selected)
					.on_toggle(Message::ProjectListSelectAllProjects),
				column(project_list)
					.spacing(10)
			]
				.spacing(10)
		)
			.width(Length::Fill)
			.into()
	}

	fn view_project_list(&self) -> Element<Message> {
		column![
			heading("Projects"),
			container(
				button("New Project").on_press(Message::NewProject),
			)
			.width(Length::Fill)
			.align_x(Right),
			self.project_list()
		]
			.spacing(20)
			.padding(20)
			.into()
	}

	fn view_edit_project(&self) -> Element<Message> {
		match &self.current_project {
			None => {
				return text("No project found").into();
			},
			Some(project) => {
				match &self.project_form_state {
					None => {
						return text("No project form found").into();
					},
					Some(project_form_state) => {
						container(
							column![
								heading("New Project"),
								project_form(project, &project_form_state),
								container(
									row![
										button("Save").on_press(Message::CurrentProjectSave),
										button("Cancel").on_press(Message::EditProjectCancel),
									]
									.spacing(5)
								)
								.width(Length::Fill)
								.align_x(Right)
							]
							.spacing(20)
						)
						.padding(20)
						.into()
					}
				}
			}
		}
	}

}

impl App {
	fn new() -> (Self, Task<Message>) {
		(
			Self::Loading,
			Task::perform(async {
				AppState::load().await.map_err(|e| e.to_string())
			}, Message::AppLoaded),
		)
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match self {
			App::Loading => {
				match message {
					Message::AppLoaded(Ok(state)) => {
						*self = App::Loaded(AppState {
							projects_data: state.projects_data,
							context: Some(Context::ProjectList),
							..AppState::default()
						});
					}
					Message::AppLoaded(Err(_)) => {
						*self = App::Loaded(AppState::default());
					}
					_ => {}
				}

				Task::none()
			},
			App::Loaded(state) => {
				match message {
					Message::AppSync => {
						state.save().unwrap();

						Task::none()
					},
					Message::NewProject => {
						let name = "".to_string();
						let description = "".to_string();
						state.current_project = Some(Project::new(&name, &description));
						state.project_form_state = Some(ProjectFormState::new());
						state.context = Some(Context::NewProject);

						focus_next()
					},
					Message::EditProject(project_id) => {
						let projects_data = state.projects_data.as_ref();
						if let Some(projects_data) = projects_data {
							state.current_project = projects_data.get_project(&project_id).cloned();
							state.project_form_state = Some(ProjectFormState::new());
							state.context = Some(Context::EditProject);
						}

						Task::none()
					},
					Message::EditProjectCancel => {
						state.current_project = None;
						state.context = Some(Context::ProjectList);

						Task::none()
					},
					Message::CurrentProjectNameChange(name) => {
						match &mut state.current_project {
							None => {},
							Some(project) => {
								project.name = name;

								let project = state.current_project.as_ref().unwrap();

								match &mut state.project_form_state {
									None => {},
									Some(form) => {
										let name_field = &mut form.name_field;

										if project.name.len() == 0 {
											name_field.is_valid = false;
											name_field.validation_message = "Name is required".to_string();
										} else {
											name_field.is_valid = true;
											name_field.validation_message = "".to_string();
										}
									}
								}
							}
						}

						Task::none()
					},
					Message::CurrentProjectDescriptionChange(description) => {
						match &mut state.current_project {
							None => {},
							Some(project) => {
								project.description = description;
							}
						}

						Task::none()
					},
					Message::CurrentProjectSave => {
						state.save_current_project();

						Task::none()
					},
					Message::ProjectListProjectSelected {selected, selected_project_id} => {
						match &mut state.projects_data {
							None => {},
							Some(_) => {
								match &mut state.project_list_state {
									None => {},
									Some(project_list_state) => {
										if selected {
											project_list_state.selected_projects.push(selected_project_id);
										} else {
											project_list_state.selected_projects.retain(|project_id| project_id != &selected_project_id);
										}
									}
								}
							}
						}

						Task::none()
					},
					Message::ProjectListSelectAllProjects(selected) => {
						let project_ids: Vec<Uuid> = if selected {
							state.get_projects().iter().map(|project| project.id).collect()
						} else {
							Vec::new()
						};

						if let Some(project_list_state) = &mut state.project_list_state {
							project_list_state.selected_projects = project_ids;
						}

						Task::none()
					},
					_ => Task::none(),
				}
			}
		}
	}

	fn view(&self) -> Element<Message> {
		match self {
			App::Loading => {
				return text("Loading...").into();
			},
			App::Loaded(state) => {
				return state.view();
			}
		}
	}

	fn subscription(&self) -> Subscription<Message> {
		let tick = time::every(Duration::from_secs(15)).map(|_| {
			Message::AppSync
		});

		Subscription::batch(vec![tick])
	}
}

pub fn main() -> iced::Result {
	iced::application("Task Manager", App::update, App::view)
		.subscription(App::subscription)
		.run_with(App::new)
}