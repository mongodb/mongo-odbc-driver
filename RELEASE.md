# Releasing the MongoDB Atlas SQL interface ODBC Driver

This document describes the version policy and release process for the MongoDB Atlas SQL interface ODBC Driver.

## Versioning

Versions will follow the [semantic versioning](https://semver.org/) system.

The following guidelines will be used to determine when each version component will be updated:
- **major**: backwards-breaking changes
- **minor**: functionality added in a backwards compatible manner
- **patch**: backwards compatible bug fixes.
- **pre-release**: The pre-release version, used for preview builds

## Release Process

### Pre-Release Tasks

#### Determine the correct version to be released

Go to the [SQL releases page](https://jira.mongodb.org/projects/SQL?selectedItem=com.atlassian.jira.jira-projects-plugin%3Arelease-page&status=unreleased), and check the content of the tickets that are included in the current release. The fix version by default is a patch version. If there is a backwards incompatible API change in the tickets that are set to be released, we should instead update the major version; if there are new features added in the tickets set to be released, we should instead update the minor version. To do so, update the version on the [SQL releases page](https://jira.mongodb.org/projects/SQL?selectedItem=com.atlassian.jira.jira-projects-plugin%3Arelease-page&status=unreleased) under "Actions". This will update the fix version on all of the tickets as well.

##### Determine the correct version of `mongosql` to tag in `odbc/cargo.toml`. Cargo will use this specific tag.

#### Start Release Ticket
Move the JIRA ticket for the release to the "In Progress" state.
Ensure that its fixVersion matches the version being released, and update it if it changed in the previous step.

Add the fixVersion for the version of `mongosql-rs` that will be bundled as a separate fix version to the ticket.

#### Complete the Release in JIRA
Go to the [SQL releases page](https://jira.mongodb.org/projects/SQL?selectedItem=com.atlassian.jira.jira-projects-plugin%3Arelease-page&status=unreleased), and ensure that all the tickets in the fixVersion to be released are closed.
Ensure that all the tickets have the correct type. Take this opportunity to edit ticket titles if they can be made more descriptive.
The ticket titles will be published in the changelog.

The only uncompleted ticket in the release should be the release ticket.
If there are any remaining tickets that will not be included in this release, remove the fixVersion and assign them a new one if appropriate.

Close the release on JIRA, adding the current date (you may need to ask the SQL project manager to do this).

#### Ensure Evergreen Passing
Ensure that the build you are releasing is passing the tests on the evergreen waterfall.

### Release Tasks

#### Ensure master up to date
Ensure you have the `master` branch checked out, and that you have pulled the latest commit from `mongodb/mongo-odbc-driver`.

#### Create the tag and push

`v<ODBC major>.<ODBC minor>.<ODBC patch>[-<prereleaseversion>]`

Create an annotated tag and push it:

```sh
#git tag -am X.Y.Z[-<ODBCprerelease>]
git tag -am X.Y.Z-alpha-1 vX.Y.Z-alpha-1
git push --tags
```

This should trigger an Evergreen version that can be viewed on the [mongo-odbc-driver waterfall](https://evergreen.mongodb.com/waterfall/mongosql-odbc-driver).
If it does not, you may have to ask the project manager to give you the right permissions to do so.
Make sure to run the 'release' task, if it is not run automatically.

#### Set Evergreen Priorities

Some evergreen variants may have a long schedule queue.
To speed up release tasks, you can set the task priority for any variant to 101 for release candidates and 200 for actual releases.
If you do not have permissions to set priority above 100, ask the project manager to set the
priority.

### Post-Release Tasks

#### Wait for evergreen

Wait for the evergreen version to finish, and ensure that the release task completes successfully.

#### Verify release artifacts

Check that the released files, library and symbols, are available at the following URLs:
- Windows
  - Release build
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/eap/mongosql-odbc-driver/windows/${release_version}/release/atsql.dll`
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/eap/mongosql-odbc-driver/windows/${release_version}/release/atsqls.dll`
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/eap/mongosql-odbc-driver/windows/${release_version}/release/atsql.pdb`
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/eap/mongosql-odbc-driver/windows/${release_version}/release/mongoodbc.msi`
- Ubuntu 2204
  - Release build
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/eap/mongosql-odbc-driver/ubuntu2204/${release_version}/release/libatsql.so`
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/eap/mongosql-odbc-driver/ubuntu2204/${release_version}/release/mongoodbc.tar.gz`

- Documentation
  - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/eap/mongodb-odbc-driver/docs/MongoDB_ODBC_Guide.pdf`

During the EAP, the following URLs are not used. They are retained in this document for legacy purposes, and will become correct again
once the EAP completes.

Check that the released files, library and symbols, are available at the following URLs:
- Windows
  - Release build
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/mongosql-odbc-driver/windows/${release_version}/release/atsql.dll`
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/mongosql-odbc-driver/windows/${release_version}/release/atsqls.dll`
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/mongosql-odbc-driver/windows/${release_version}/release/atsql.pdb`
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/mongosql-odbc-driver/windows/${release_version}/release/mongoodbc.msi`
- Ubuntu 2204
  - Release build
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/mongosql-odbc-driver/ubuntu2204/${release_version}/release/libatsql.so`
    - `https://translators-connectors-releases.s3.us-east-1.amazonaws.com/mongosql-odbc-driver/ubuntu2204/${release_version}/release/mongoodbc.tar.gz`

##### Verify that the driver works with PowerBI

Download and install the driver file.

Verify that it is able to connect to Atlas Data Federation with PowerBI, extract data,
and add columns to the worksheet.

#### Close Release Ticket

Move the JIRA ticket tracking this release to the "Closed" state.

#### Ensure next release ticket and fixVersion created

Ensure that a JIRA ticket tracking the next release has been created and is assigned the appropriate fixVersion. The fixVersion should
contain the `mongosql-rs` version. 
