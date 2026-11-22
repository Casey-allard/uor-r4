// The apps Foundry shows every visitor on Home, the same four for everyone: the floor shows the first
// four of this list, and the others wait their turn. Each is a Prism app: its link is
// filled in as it is published, and until then its card opens its idea in the box. The cover is drawn from
// the card's own hue (shell/appearance.css keeps the page's colours in tokens; a cover's light is its own
// subject, so it is written here, beside the app it belongs to).
export const featuredOnHome = 4;

export const featuredApps = [
  {
    id: "pantry-recipe-planner",
    by: "UOR Foundation",
    name: "Pantry recipe planner",
    sentence: "Tells you what you can cook from what is already in the cupboard.",
    tag: "Home",
    idea: "Build a pantry recipe planner that suggests meals from what I already have",
    hue: 18,
    url: "",
  },
  {
    id: "tiny-team-crm",
    by: "UOR Foundation",
    name: "Tiny team CRM",
    sentence: "Keeps every conversation with a customer in one place, for a team of three.",
    tag: "Work",
    idea: "Build a simple CRM for a team of three, with contacts, notes and next steps",
    hue: 196,
    url: "",
  },
  {
    id: "workout-streak-coach",
    by: "UOR Foundation",
    name: "Workout streak coach",
    sentence: "Counts the days you turned up and shows the streak you are protecting.",
    tag: "Health",
    idea: "Build a workout streak tracker that shows my current streak and best week",
    hue: 142,
    url: "",
  },
  {
    id: "apartment-hunt-board",
    by: "UOR Foundation",
    name: "Apartment hunt board",
    sentence: "Lines up every flat you have seen so the trade-offs are visible at once.",
    tag: "Life",
    idea: "Build an apartment hunting board comparing rent, commute and notes per flat",
    hue: 268,
    url: "",
  },
  {
    id: "music-discovery-log",
    by: "UOR Foundation",
    name: "Music discovery log",
    sentence: "Remembers what you heard, where you heard it and what it reminded you of.",
    tag: "Culture",
    idea: "Build a music discovery log with what I heard, where, and how it felt",
    hue: 324,
    url: "",
  },
  {
    id: "travel-memory-map",
    by: "UOR Foundation",
    name: "Travel memory map",
    sentence: "Pins each trip to the place it happened, with the photographs beside it.",
    tag: "Travel",
    idea: "Build a travel memory map pinning trips to places with photos and notes",
    hue: 44,
    url: "",
  },
];
