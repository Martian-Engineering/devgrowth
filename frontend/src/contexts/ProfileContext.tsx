// src/contexts/ProfileContext.tsx
import React, {
  createContext,
  useContext,
  useReducer,
  ReactNode,
  useEffect,
  useCallback,
} from "react";
import { fetchWrapper } from "@/lib/fetchWrapper";

export interface ProfileData {
  github_id: string;
  login: string;
  name: string | null;
  email: string | null;
  starred_repositories: GithubRepo[];
  repo_collections: RepoCollectionMap;
}

export interface GithubRepo {
  id: number;
  name: string;
  owner: string;
  html_url: string;
  description: string | null;
  stargazers_count: number | null;
  synced_at: Date | null;
}

export interface RepoCollectionMap {
  [repoId: number]: number[];
}

type ProfileAction =
  | { type: "SET_PROFILE_DATA"; payload: ProfileData }
  | {
      type: "ADD_REPOSITORY_TO_COLLECTION";
      payload: { repoId: number; collectionId: number };
    }
  | {
      type: "REMOVE_REPOSITORY_FROM_COLLECTION";
      payload: { repoId: number; collectionId: number };
    };

interface ProfileContextType {
  profileData: ProfileData | null;
  dispatch: React.Dispatch<ProfileAction>;
  refetchProfile: () => void;
}

const ProfileContext = createContext<ProfileContextType | undefined>(undefined);

function profileReducer(
  state: ProfileData | null,
  action: ProfileAction,
): ProfileData | null {
  switch (action.type) {
    case "SET_PROFILE_DATA":
      return action.payload;
    case "ADD_REPOSITORY_TO_COLLECTION":
      if (!state) return null;
      return {
        ...state,
        repo_collections: {
          ...state.repo_collections,
          [action.payload.repoId]: [
            ...(state.repo_collections[action.payload.repoId] || []),
            action.payload.collectionId,
          ],
        },
      };
    case "REMOVE_REPOSITORY_FROM_COLLECTION":
      if (!state) return null;
      return {
        ...state,
        repo_collections: {
          ...state.repo_collections,
          [action.payload.repoId]: state.repo_collections[
            action.payload.repoId
          ].filter((id) => id !== action.payload.collectionId),
        },
      };
    default:
      return state;
  }
}

export function ProfileProvider({ children }: { children: ReactNode }) {
  const [profileData, dispatch] = useReducer(profileReducer, null);

  const fetchProfileData = useCallback(() => {
    fetchWrapper("/api/account/profile", {
      credentials: "include",
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error("Failed to fetch profile data");
        }
        return response.json();
      })
      .then((data) => dispatch({ type: "SET_PROFILE_DATA", payload: data }))
      .catch((error) => console.error("Error fetching profile data:", error));
  }, []);

  useEffect(() => {
    fetchProfileData();
  }, [fetchProfileData]);

  return (
    <ProfileContext.Provider
      value={{ profileData, dispatch, refetchProfile: fetchProfileData }}
    >
      {children}
    </ProfileContext.Provider>
  );
}

export function useProfile() {
  const context = useContext(ProfileContext);
  if (context === undefined) {
    throw new Error("useProfile must be used within a ProfileProvider");
  }
  return context;
}
