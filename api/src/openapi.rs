use utoipa::{Modify, OpenApi};

use domain::models::{
    AuthLoginRequest, AuthRegisterRequest, Education, EducationKeyPoint, Framework, Language,
    NewEducationKeyPointRequest, NewEducationRequest, NewFrameworkRequest, NewLanguageRequest,
    NewPortfolioKeyPointRequest, NewPortfolioProjectRequest, NewPortfolioTechnologyRequest,
    NewResumeRequest, NewSkillRequest, NewWorkExperienceKeyPointRequest, NewWorkExperienceRequest,
    PortfolioKeyPoint, PortfolioProject, PortfolioTechnology, Resume, Skill, UpdateEducation,
    UpdateEducationKeyPoint, UpdateFramework, UpdateLanguage, UpdatePortfolioKeyPoint,
    UpdatePortfolioProject, UpdatePortfolioTechnology, UpdateResume, UpdateSkill,
    UpdateWorkExperience, UpdateWorkExperienceKeyPoint, User, WorkExperience,
    WorkExperienceKeyPoint,
};
use shared::response_models::{AuthTokenResponse, Response};
use utoipa::openapi::ComponentsBuilder;
use utoipa::openapi::Server;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.take().unwrap_or_default();
        openapi.components = Some(
            ComponentsBuilder::from(components)
                .security_scheme(
                    "bearerAuth",
                    SecurityScheme::Http(
                        HttpBuilder::new()
                            .scheme(HttpAuthScheme::Bearer)
                            .bearer_format("UUID")
                            .build(),
                    ),
                )
                .build(),
        );
    }
}

pub struct ServerAddon;

impl Modify for ServerAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/api")]);
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon, &ServerAddon),
    paths(
        crate::auth_handler::register_handler,
        crate::auth_handler::login_handler,
        crate::auth_handler::me_handler,
        crate::auth_handler::logout_handler,
        crate::ws_handler::ws_handler,
        crate::resume_handler::list_resumes_handler,
        crate::resume_handler::list_resume_handler,
        crate::resume_handler::create_resume_handler,
        crate::resume_handler::update_resume_handler,
        crate::resume_handler::delete_resume_handler,
        crate::skills_handler::list_skills_handler,
        crate::skills_handler::create_skill_handler,
        crate::skills_handler::update_skill_handler,
        crate::skills_handler::delete_skill_handler,
        crate::languages_handler::list_languages_handler,
        crate::languages_handler::create_language_handler,
        crate::languages_handler::update_language_handler,
        crate::languages_handler::delete_language_handler,
        crate::frameworks_handler::list_frameworks_handler,
        crate::frameworks_handler::create_framework_handler,
        crate::frameworks_handler::update_framework_handler,
        crate::frameworks_handler::delete_framework_handler,
        crate::education_handler::list_educations_handler,
        crate::education_handler::create_education_handler,
        crate::education_handler::update_education_handler,
        crate::education_handler::delete_education_handler,
        crate::education_handler::list_education_key_points_handler,
        crate::education_handler::create_education_key_point_handler,
        crate::education_handler::update_education_key_point_handler,
        crate::education_handler::delete_education_key_point_handler,
        crate::work_experiences_handler::list_work_experiences_handler,
        crate::work_experiences_handler::create_work_experience_handler,
        crate::work_experiences_handler::update_work_experience_handler,
        crate::work_experiences_handler::delete_work_experience_handler,
        crate::work_experiences_handler::list_work_experience_key_points_handler,
        crate::work_experiences_handler::create_work_experience_key_point_handler,
        crate::work_experiences_handler::update_work_experience_key_point_handler,
        crate::work_experiences_handler::delete_work_experience_key_point_handler,
        crate::portfolio_projects_handler::list_portfolio_projects_handler,
        crate::portfolio_projects_handler::create_portfolio_project_handler,
        crate::portfolio_projects_handler::update_portfolio_project_handler,
        crate::portfolio_projects_handler::delete_portfolio_project_handler,
        crate::portfolio_projects_handler::list_portfolio_key_points_handler,
        crate::portfolio_projects_handler::create_portfolio_key_point_handler,
        crate::portfolio_projects_handler::update_portfolio_key_point_handler,
        crate::portfolio_projects_handler::delete_portfolio_key_point_handler,
        crate::portfolio_projects_handler::list_portfolio_technologies_handler,
        crate::portfolio_projects_handler::create_portfolio_technology_handler,
        crate::portfolio_projects_handler::update_portfolio_technology_handler,
        crate::portfolio_projects_handler::delete_portfolio_technology_handler,
    ),
    components(schemas(
        AuthTokenResponse,
        Response::<User>,
        Response::<Resume>,
        Response::<Skill>,
        Response::<Language>,
        Response::<Framework>,
        Response::<Education>,
        Response::<EducationKeyPoint>,
        Response::<WorkExperience>,
        Response::<WorkExperienceKeyPoint>,
        Response::<PortfolioProject>,
        Response::<PortfolioKeyPoint>,
        Response::<PortfolioTechnology>,
        Response::<AuthRegisterRequest>,
        Response::<AuthLoginRequest>,
        Response::<NewResumeRequest>,
        Response::<UpdateResume>,
        Response::<NewSkillRequest>,
        Response::<UpdateSkill>,
        Response::<NewLanguageRequest>,
        Response::<UpdateLanguage>,
        Response::<NewFrameworkRequest>,
        Response::<UpdateFramework>,
        Response::<NewEducationRequest>,
        Response::<UpdateEducation>,
        Response::<NewEducationKeyPointRequest>,
        Response::<UpdateEducationKeyPoint>,
        Response::<NewWorkExperienceRequest>,
        Response::<UpdateWorkExperience>,
        Response::<NewWorkExperienceKeyPointRequest>,
        Response::<UpdateWorkExperienceKeyPoint>,
        Response::<NewPortfolioProjectRequest>,
        Response::<UpdatePortfolioProject>,
        Response::<NewPortfolioKeyPointRequest>,
        Response::<UpdatePortfolioKeyPoint>,
        Response::<NewPortfolioTechnologyRequest>,
        Response::<UpdatePortfolioTechnology>,
    ))
)]
pub struct ApiDoc;
