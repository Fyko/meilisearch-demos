# Demo Update Process

## Overview

Updating a demo to the most recent version causes a number of issues. As a result, the focus of this process is to document all of the steps involved in migrating a demo from an older version of Meilisearch to the most recent version.

## What to do:

- Clone the Demos repo's main branch to locally work on any upgrade.

> 🙂 Don’t work on the production repo

- Create a new branch for each demo that needs to be updated to start making changes locally. As a result, each demo will have its own PR, making the code review process easier.
- Update the demo and test it locally to ensure everything is working properly.
- Create a PR and request a developer advocate to review the code changes. Don’t forget to highlight breaking changes in the PR:
    - Meilisearch API routes, as well as functions in the Meilisearch SDK that may have changed;
    - New feature added to the demo if any;
    - Attach the link to the full changelog to be clear about the new feature(s) and the impact on the users.
- After the code review is approved, merge the PR(s).
- Follow this [guide](https://github.com/meilisearch/meilisearch-migration) to update the corresponding demo server in DigitalOcean. You can find demo droplets and their URLs [here](https://www.notion.so/5de77184bb8e42f8a1baa95dec76ec33) *(private link)*.
- Because new values for the `Default Search API Key` and `Default Admin API Key` are generated after updating Meilisearch on DigitalOcean:
    - Update breaking changes note with the new `Default Search API Key` value;
    - Update these changes in the code/env file;
    - Communicate the new API keys to whoever owns the demo sandboxes.

    