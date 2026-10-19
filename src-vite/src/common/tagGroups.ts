export interface TagGroup {
  id: number;
  name: string;
  is_default: boolean;
  count: number;
}
export interface LibraryTag {
  id: number;
  name: string;
  group_id: number;
  group_name: string;
  count?: number;
}
export function groupTags(
  groups: TagGroup[],
  tags: LibraryTag[],
  search: string,
) {
  const query = search.trim().toLocaleLowerCase();
  const members = new Map<number, LibraryTag[]>();
  for (const tag of tags) {
    if (!members.has(tag.group_id)) members.set(tag.group_id, []);
    members.get(tag.group_id)!.push(tag);
  }
  return groups
    .map((group) => {
      const all = members.get(group.id) || [];
      const match = group.name.toLocaleLowerCase().includes(query);
      return {
        ...group,
        tags: match
          ? all
          : all.filter((tag) => tag.name.toLocaleLowerCase().includes(query)),
        matchesGroup: match,
      };
    })
    .filter((group) => !query || group.matchesGroup || group.tags.length > 0);
}
