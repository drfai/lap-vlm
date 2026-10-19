export interface SmartTagDef {
  id: string;
  // CLIP text prompt should stay in English for stable semantic search.
  prompt: string;
  // Override only after calibrating this tag against real library results.
  threshold?: number;
}

export interface SmartTagCategoryDef {
  id: string;
  items: SmartTagDef[];
}

export const SMART_TAG_CATEGORIES: SmartTagCategoryDef[] = [
  { id: 'family', items: [{ id: 'family', prompt: 'a group photo of multiple people of different ages including adults and children, family gathering or reunion' }] },
  { id: 'kids', items: [{ id: 'kids', prompt: 'a photo of young children or infant baby, toddler playing or smiling' }] },
  { id: 'pets', items: [{ id: 'pets', prompt: 'a close-up or indoor photo of a domestic pet dog or cat, animal portrait' }] },
  { id: 'portraits', items: [{ id: 'portraits', prompt: 'a portrait or headshot of a single person, face clearly visible, shallow depth of field background blur' }] },
  { id: 'food', items: [{ id: 'food', prompt: 'a top-down or close-up food photography of a plated dish or meal, restaurant or home cooking' }] },
  { id: 'sports', items: [{ id: 'sports', prompt: 'an action sports photo with motion blur or dynamic pose, people running jumping cycling or competing outdoors' }] },
  { id: 'landscape', items: [{ id: 'landscape', prompt: 'a scenic landscape photo of nature, mountains, valley, ocean coast, forest, or open sky, no people' }] },
  { id: 'night', items: [{ id: 'night', prompt: 'a nighttime or low-light photo with city lights, neon signs, street lights, or starry night sky' }] },
];

export function getSmartTagById(id: string | null | undefined): SmartTagDef | null {
  if (!id) return null;
  for (const category of SMART_TAG_CATEGORIES) {
    const found = category.items.find(item => item.id === id);
    if (found) return found;
  }
  return null;
}
