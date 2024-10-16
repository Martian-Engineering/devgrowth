import { RepoCollectionMap } from "@/contexts/ProfileContext";

export function getRepositoryCountForCollection(
  collectionId: number,
  repoCollections: RepoCollectionMap,
): number {
  return Object.values(repoCollections).reduce((count, collectionIds) => {
    if (collectionIds.includes(collectionId)) {
      return count + 1;
    }
    return count;
  }, 0);
}
