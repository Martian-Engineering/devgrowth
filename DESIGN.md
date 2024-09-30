Analyze the developer growth of any GitHub repository. Group repositories into
collections and examine them in aggregate.

First, we need an endpoint that accepts a GitHub repo via a POST and enqueues it
for analysis.

Analyzing a repository means pulling all of its commits from the GitHub API and
writing them into a Postgres database. If the repository has never before been
seen, that means the entire repo history. If it has been seen, only the commits
since the most recent stored one need be queried. This operation is expensive
and likely to be rate-limited by the GitHub API, so it should space requests out
to avoid rate limiting, and, when limited, back off and retry until rate limits
expire.
