use crate::helpers::{test_context::TestContext, TEST_REGISTRY};

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_adds_changelog_on_new_project() {
    let context = TestContext::new().await;

    context.run_release_pr().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);

    let changed_files = context
        .gitea
        .changed_files_in_pr(opened_prs[0].number)
        .await;
    assert_eq!(changed_files.len(), 1);
    assert_eq!(changed_files[0].filename, "CHANGELOG.md");
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_releases_a_new_project() {
    let context = TestContext::new().await;

    let crate_name = &context.gitea.repo;
    let dest_dir = tempfile::tempdir().unwrap();
    let dest_dir_str = dest_dir.path().to_str().unwrap();

    let packages = || {
        release_plz_core::PackageDownloader::new([crate_name], dest_dir_str)
            .with_registry(TEST_REGISTRY.to_string())
            .with_cargo_cwd(context.repo_dir())
            .download()
            .unwrap()
    };
    // Before running release-plz, no packages should be present.
    assert!(packages().is_empty());

    context.run_release().success();

    assert_eq!(packages().len(), 1);
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_adds_custom_changelog() {
    let context = TestContext::new().await;
    let config = r#"
    [changelog]
    header = "Changelog\n\n"
    body = """
    == [{{ version }}]
    {% for group, commits in commits | group_by(attribute="group") %}
    === {{ group | upper_first }}
    {% for commit in commits %}
    {%- if commit.scope -%}
    - *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}{%- if commit.links %} ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%}){% endif %}
    {% else -%}
    - {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}
    {% endif -%}
    {% endfor -%}
    {% endfor %}"
    """
    trim = true
    "#;
    context.write_release_plz_toml(config);

    context.run_release_pr().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);

    let changelog = context
        .gitea
        .get_file_content(opened_prs[0].branch(), "CHANGELOG.md")
        .await;
    expect_test::expect![[r#"
        Changelog

        == [0.1.0]

        === Other
        - add config file
        - cargo init
        - Initial commit
        "
    "#]]
    .assert_eq(&changelog);
}
