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
import { Repository, parseGithubRepos } from "@/lib/repository";

export interface Profile {
  account?: Account;
  starred_repositories?: Repository[];
  repo_collections?: RepoCollectionMap;
  collections?: Collection[];
}

export interface Account {
  github_id: string;
  login: string;
  name: string | null;
  email: string | null;
}

export interface RepoCollectionMap {
  [repoId: number]: number[];
}

export interface Collection {
  collection_id: number;
  name: string;
  description: string;
  repositories: Repository[];
}

type ProfileAction =
  // Internal
  | { type: "SET_ACCOUNT"; payload: Account }
  | { type: "SET_STARRED_REPOS"; payload: Repository[] }
  | { type: "SET_REPO_COLLECTIONS"; payload: RepoCollectionMap }
  | { type: "SET_COLLECTIONS"; payload: Collection[] }
  // External
  | { type: "SET_COLLECTION"; payload: Collection }
  | { type: "CREATE_COLLECTION"; payload: Collection }
  // | { type: 'UPDATE_COLLECTION'; payload: Collection }
  // | { type: 'DELETE_COLLECTION'; payload: number }
  | {
      type: "ADD_REPOSITORY_TO_COLLECTION";
      payload: {
        repoId: number | null;
        collectionId: number;
        repository?: Repository;
      };
    }
  | {
      type: "REMOVE_REPOSITORY_FROM_COLLECTION";
      payload: { repoId: number; collectionId: number };
    };

interface ProfileContextType {
  profile: Profile | null;
  dispatch: React.Dispatch<ProfileAction>;
  refetchProfile: () => void;
  refetchAccount: () => void;
  refetchRepoCollections: () => void;
  refetchStarredRepos: () => void;
  refetchCollections: () => void;
}

const ProfileContext = createContext<ProfileContextType | undefined>(undefined);

function profileReducer(
  state: Profile | null,
  action: ProfileAction,
): Profile | null {
  switch (action.type) {
    case "SET_ACCOUNT":
      return {
        ...state,
        account: action.payload,
      };
    case "SET_REPO_COLLECTIONS":
      return {
        ...state,
        repo_collections: action.payload,
      };
    case "SET_STARRED_REPOS":
      return {
        ...state,
        starred_repositories: action.payload,
      };
    case "SET_COLLECTIONS":
      return {
        ...state,
        collections: action.payload,
      };
    case "SET_COLLECTION":
      if (!state || !state.collections) return state;
      return {
        ...state,
        collections: [
          ...state.collections.map((collection) =>
            collection.collection_id === action.payload.collection_id
              ? action.payload
              : collection,
          ),
        ],
      };
    case "CREATE_COLLECTION":
      if (!state || !state.collections) return state;
      return {
        ...state,
        collections: [action.payload, ...state.collections],
      };
    case "ADD_REPOSITORY_TO_COLLECTION":
      if (!state || !state.collections || !state.repo_collections) return state;
      const addRepoId = action.payload.repoId;
      if (addRepoId === null) return state;

      const updatedCollections = state.collections.map((collection) => {
        if (collection.collection_id === action.payload.collectionId) {
          return {
            ...collection,
            repositories: [
              ...collection.repositories,
              action.payload.repository || {
                repository_id: addRepoId,
                name: "",
                owner: "",
                indexed_at: null,
                description: null,
                stargazers_count: 0,
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
              },
            ],
          };
        }
        return collection;
      });

      return {
        ...state,
        repo_collections: {
          ...state.repo_collections,
          [addRepoId]: [
            ...(state.repo_collections[addRepoId] || []),
            action.payload.collectionId,
          ],
        },
        collections: updatedCollections,
      };
    case "REMOVE_REPOSITORY_FROM_COLLECTION":
      if (!state || !state.collections || !state.repo_collections) return null;
      const removeRepoId = action.payload.repoId;
      if (removeRepoId === null) return state;
      const updatedCollectionsAfterRemove = state.collections.map(
        (collection) => {
          if (collection.collection_id === action.payload.collectionId) {
            return {
              ...collection,
              repositories: collection.repositories.filter(
                (repo) => repo.repository_id !== removeRepoId,
              ),
            };
          }
          return collection;
        },
      );

      return {
        ...state,
        repo_collections: {
          ...state.repo_collections,
          [removeRepoId]: state.repo_collections[removeRepoId].filter(
            (id) => id !== action.payload.collectionId,
          ),
        },
        collections: updatedCollectionsAfterRemove,
      };
    default:
      return state;
  }
}

export function ProfileProvider({ children }: { children: ReactNode }) {
  const [profile, dispatch] = useReducer(profileReducer, null);

  const fetchAccount = useCallback(() => {
    fetchWrapper("/api/account/profile")
      .then((response) => response.json())
      .then((data) => dispatch({ type: "SET_ACCOUNT", payload: data }))
      .catch((error) => console.error("Error fetching account:", error));
  }, []);

  const fetchRepoCollections = useCallback(() => {
    fetchWrapper("/api/account/repo-collections")
      .then((response) => response.json())
      .then((data) => dispatch({ type: "SET_REPO_COLLECTIONS", payload: data }))
      .catch((error) =>
        console.error("Error fetching repo collections:", error),
      );
  }, []);

  const fetchStarredRepos = useCallback(() => {
    fetchWrapper("/api/github/starred")
      .then((response) => response.json())
      .then((data) =>
        dispatch({
          type: "SET_STARRED_REPOS",
          payload: parseGithubRepos(data),
        }),
      )
      .catch((error) => console.error("Error fetching starred repos:", error));
  }, []);

  const fetchCollections = useCallback(() => {
    fetchWrapper("/api/collections")
      .then((response) => response.json())
      .then((data) => dispatch({ type: "SET_COLLECTIONS", payload: data }))
      .catch((error) => console.error("Error fetching collections:", error));
  }, []);

  const fetchAllProfileData = useCallback(() => {
    fetchAccount();
    fetchRepoCollections();
    fetchStarredRepos();
    fetchCollections();
  }, [fetchAccount, fetchRepoCollections, fetchStarredRepos, fetchCollections]);

  useEffect(() => {
    fetchAllProfileData();
  }, [fetchAllProfileData]);

  return (
    <ProfileContext.Provider
      value={{
        profile,
        dispatch,
        refetchProfile: fetchAllProfileData,
        refetchAccount: fetchAccount,
        refetchRepoCollections: fetchRepoCollections,
        refetchStarredRepos: fetchStarredRepos,
        refetchCollections: fetchCollections,
      }}
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
