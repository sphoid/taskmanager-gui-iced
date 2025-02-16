use iced::widget::{button, checkbox, column, container, focus_next, row, text, text_input, Row};
use iced::{Element, Length, Right, Task};
use uuid::Uuid;
use taskmanager::project;
use taskmanager::project::{ProjectData, Project};

enum Context {
	ProjectList,
	NewProject,
}

struct ProjectListState {
	selected_projects: Vec<Uuid>,
}

struct FormFieldState {
	is_valid: bool,
	validation_message: String,
}

struct ProjectFormState {
	name_field: FormFieldState,
	description_field: FormFieldState,
}

struct App {
	context: Context,
	projects_data: ProjectData,
	current_project: Option<Project>,
	project_list_state: ProjectListState,
	project_form_state: ProjectFormState,
}

#[derive(Debug, Clone)]
enum Message {
	NewProject,
	NewProjectCancel,
	CurrentProjectAdd,
	CurrentProjectNameChange(String),
	CurrentProjectDescriptionChange(String),
	ProjectListProjectSelected { selected: bool, selected_project_id: Uuid },
	ProjectListSelectAllProjects(bool),
}

impl Default for App {
	fn default() -> Self {
		Self {
			context: Context::ProjectList,
			projects_data: project::load_data().unwrap(),
			current_project: None,
			project_list_state: ProjectListState {
				selected_projects: Vec::new(),
			},
			project_form_state: ProjectFormState {
				name_field: FormFieldState {
					is_valid: false,
					validation_message: "".to_string(),
				},
				description_field: FormFieldState {
					is_valid: false,
					validation_message: "".to_string(),
				},
			},
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

impl App {
	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::NewProject => {
				let name = "".to_string();
				let description = "".to_string();
				self.current_project = Some(Project::new(&name, &description));
				self.project_form_state = ProjectFormState {
					name_field: FormFieldState {
						is_valid: false,
						validation_message: "".to_string(),
					},
					description_field: FormFieldState {
						is_valid: false,
						validation_message: "".to_string(),
					},
				};
				self.context = Context::NewProject;

				focus_next()
			},
			Message::NewProjectCancel => {
				self.current_project = None;
				self.context = Context::ProjectList;

				Task::none()
			},
			Message::CurrentProjectNameChange(name) => {
				match &mut self.current_project {
					None => {},
					Some(project) => {
						project.name = name;

						let project = self.current_project.as_ref().unwrap();
						let name_field = &mut self.project_form_state.name_field;

						if project.name.len() == 0 {
							name_field.is_valid = false;
							name_field.validation_message = "Name is required".to_string();
						} else {
							name_field.is_valid = true;
							name_field.validation_message = "".to_string();
						}
					}
				}

				Task::none()
			},
			Message::CurrentProjectDescriptionChange(description) => {
				match &mut self.current_project {
					None => {},
					Some(project) => {
						project.description = description;
					}
				}

				Task::none()
			},
			Message::CurrentProjectAdd => {
				match &mut self.current_project {
					None => {},
					Some(project) => {
						self.projects_data.create_project(&project.name, &project.description);
						self.current_project = None;
						self.context = Context::ProjectList;
					}
				}

				Task::none()
			},
			Message::ProjectListProjectSelected {selected, selected_project_id} => {
				if selected {
					self.project_list_state.selected_projects.push(selected_project_id);
				} else {
					self.project_list_state.selected_projects.retain(|project_id| project_id != &selected_project_id);
				}

				Task::none()
			},
			Message::ProjectListSelectAllProjects(selected) => {
				if selected {
					self.project_list_state.selected_projects = self.projects_data.get_projects().iter().map(|project| {
						project.id
					}).collect();
				} else {
					self.project_list_state.selected_projects.clear();
				}

				Task::none()
			}
		}
	}

	fn project_list(&self) -> Element<Message> {
		let project_list: Vec<Element<Message>> = self.projects_data.get_projects().iter().map(|project| {
			let is_project_selected = self.project_list_state.selected_projects.contains(&project.id);

			let select_project = {
				let project = *project;
				move |selected: bool| {
					Message::ProjectListProjectSelected { selected, selected_project_id: project.id}
				}
			};

			Row::new()
				.push(checkbox("", is_project_selected).on_toggle(select_project))
				.push(text(project.name.clone()))
				.push(text(project.description.clone()))
				.spacing(15)
				.into()
		}).collect();

		if project_list.len() == 0 {
			return text("You have no projects").into();
		}

		let all_projects_selected = self.project_list_state.selected_projects.len() == self.projects_data.get_projects().len();
		let select_all_checkbox = checkbox("Select All", all_projects_selected).on_toggle(Message::ProjectListSelectAllProjects);

		container(
			column![
				select_all_checkbox,
				column(project_list)
					.spacing(10)
			].spacing(10)
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

	fn view_new_project(&self) -> Element<Message> {
		match &self.current_project {
			None => {
				return text("No project found").into();
			},
			Some(project) => {
				container(
					column![
						heading("New Project"),
						project_form(project, &self.project_form_state),
						container(
							row![
								button("Save").on_press(Message::CurrentProjectAdd),
								button("Cancel").on_press(Message::NewProjectCancel),
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

	fn view(&self) -> Element<Message> {
		match self.context {
			Context::ProjectList => {
				self.view_project_list()
			}
			Context::NewProject => {
				self.view_new_project()
			}
		}
	}
}

pub fn main() -> iced::Result {
	iced::run("Task Manager GUI", App::update, App::view)
}