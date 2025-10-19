// @generated automatically by Diesel CLI.

diesel::table! {
    education (id) {
        id -> Int4,
        resume_id -> Int4,
        #[max_length = 100]
        education_stage -> Varchar,
        #[max_length = 255]
        institution_name -> Varchar,
        #[max_length = 255]
        degree -> Varchar,
        start_date -> Date,
        end_date -> Nullable<Date>,
        description -> Nullable<Text>,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    education_key_points (id) {
        id -> Int4,
        education_id -> Int4,
        key_point -> Text,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    frameworks (id) {
        id -> Int4,
        language_id -> Int4,
        #[max_length = 255]
        framework_name -> Varchar,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    languages (id) {
        id -> Int4,
        resume_id -> Int4,
        #[max_length = 255]
        language_name -> Varchar,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    portfolio_key_points (id) {
        id -> Int4,
        portfolio_project_id -> Int4,
        key_point -> Text,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    portfolio_projects (id) {
        id -> Int4,
        resume_id -> Int4,
        #[max_length = 255]
        project_name -> Varchar,
        #[max_length = 500]
        image_url -> Nullable<Varchar>,
        #[max_length = 500]
        project_link -> Nullable<Varchar>,
        #[max_length = 500]
        source_code_link -> Nullable<Varchar>,
        description -> Nullable<Text>,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    portfolio_technologies (id) {
        id -> Int4,
        portfolio_project_id -> Int4,
        #[max_length = 255]
        technology_name -> Varchar,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    resumes (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 500]
        profile_image_url -> Nullable<Varchar>,
        #[max_length = 255]
        location -> Nullable<Varchar>,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 500]
        github_url -> Nullable<Varchar>,
        #[max_length = 50]
        mobile_number -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    skills (id) {
        id -> Int4,
        resume_id -> Int4,
        #[max_length = 255]
        skill_name -> Varchar,
        confidence_percentage -> Int4,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    work_experience_key_points (id) {
        id -> Int4,
        work_experience_id -> Int4,
        key_point -> Text,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    work_experiences (id) {
        id -> Int4,
        resume_id -> Int4,
        #[max_length = 255]
        job_title -> Varchar,
        #[max_length = 255]
        company_name -> Varchar,
        start_date -> Date,
        end_date -> Nullable<Date>,
        description -> Nullable<Text>,
        display_order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(education -> resumes (resume_id));
diesel::joinable!(education_key_points -> education (education_id));
diesel::joinable!(frameworks -> languages (language_id));
diesel::joinable!(languages -> resumes (resume_id));
diesel::joinable!(portfolio_key_points -> portfolio_projects (portfolio_project_id));
diesel::joinable!(portfolio_projects -> resumes (resume_id));
diesel::joinable!(portfolio_technologies -> portfolio_projects (portfolio_project_id));
diesel::joinable!(skills -> resumes (resume_id));
diesel::joinable!(work_experience_key_points -> work_experiences (work_experience_id));
diesel::joinable!(work_experiences -> resumes (resume_id));

diesel::allow_tables_to_appear_in_same_query!(
    education,
    education_key_points,
    frameworks,
    languages,
    portfolio_key_points,
    portfolio_projects,
    portfolio_technologies,
    resumes,
    skills,
    work_experience_key_points,
    work_experiences,
);
