// src/lib/repository.ts
export interface Repository {
  repository_id: number;
  name: string;
  owner: string;
  description: string | null;
  stargazers_count: number;
  indexed_at: string | null;
  created_at: string | null;
  updated_at: string | null;
}

export interface GithubRepo {
  id: number;
  name: string;
  owner: string;
  html_url: string;
  description: string;
  stargazers_count: number | 0;
}

export const parseGithubRepos = (repos: GithubRepo[]): Repository[] => {
  return repos.map((repo: GithubRepo) => {
    return {
      repository_id: repo.id,
      owner: repo.owner,
      name: repo.name,
      description: repo.description,
      stargazers_count: repo.stargazers_count || 0,
      created_at: null,
      updated_at: null,
      indexed_at: null,
    };
  });
};
