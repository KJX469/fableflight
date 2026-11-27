---
name: Fable Flight
description: An open-world flight simulator of the real Earth, its menus and HUD drawn in egui over the world.
colors:
  accent: "#F5C518"
  accent-hi: "#FFD648"
  on-accent: "#1A1402"
  night: "#080B10"
  surface: "#0F131A"
  raised: "#1C222C"
  raised-hi: "#2A323F"
  chosen: "#424E62"
  panel-glass: "rgba(10, 13, 19, 0.91)"
  hud-box: "rgba(11, 13, 18, 0.88)"
  hairline: "rgba(255, 255, 255, 0.10)"
  hairline-hi: "rgba(255, 255, 255, 0.17)"
  text: "#E8ECF2"
  dim: "#929CAB"
  faint: "#76808F"
  link: "#84B8FF"
  ok: "#70D68A"
  bad: "#FF7A5C"
  caution: "#FFC450"
typography:
  display:
    fontFamily: "Barlow Condensed, sans-serif"
    fontSize: "44px"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "0.04em"
  headline:
    fontFamily: "Barlow, sans-serif"
    fontSize: "28px"
    fontWeight: 600
    lineHeight: 1.15
  title:
    fontFamily: "Barlow, sans-serif"
    fontSize: "17px"
    fontWeight: 600
    lineHeight: 1.3
  label:
    fontFamily: "Barlow, sans-serif"
    fontSize: "15px"
    fontWeight: 500
    lineHeight: 1.35
  body:
    fontFamily: "Barlow, sans-serif"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1.45
  caption:
    fontFamily: "Barlow, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.4
  section:
    fontFamily: "Barlow Condensed, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    letterSpacing: "1px"
  data:
    fontFamily: "IBM Plex Mono, monospace"
    fontSize: "13px"
    fontWeight: 500
rounded:
  sm: "3px"
  none: "0px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.on-accent}"
    typography: "{typography.display}"
    rounded: "{rounded.sm}"
    height: "46px"
    padding: "0 20px"
  button-primary-hover:
    backgroundColor: "{colors.accent-hi}"
    textColor: "{colors.on-accent}"
  button:
    backgroundColor: "{colors.raised}"
    textColor: "{colors.text}"
    typography: "{typography.label}"
    rounded: "{rounded.sm}"
    height: "32px"
    padding: "0 14px"
  button-hover:
    backgroundColor: "{colors.raised-hi}"
    textColor: "#FFFFFF"
  choice-selected:
    backgroundColor: "{colors.chosen}"
    textColor: "#FFFFFF"
    rounded: "{rounded.sm}"
    height: "32px"
  panel:
    backgroundColor: "{colors.panel-glass}"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: "24px"
  card:
    backgroundColor: "{colors.panel-glass}"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: "16px 24px"
  hud-panel:
    backgroundColor: "{colors.hud-box}"
    textColor: "#FFFFFF"
    rounded: "{rounded.none}"
    padding: "8px 16px"
  key-cap:
    backgroundColor: "#3A4454"
    textColor: "#FFFFFF"
    typography: "{typography.label}"
    rounded: "{rounded.sm}"
    height: "24px"
    padding: "0 8px"
  tab:
    textColor: "{colors.dim}"
    typography: "{typography.label}"
    height: "40px"
  tab-active:
    textColor: "#FFFFFF"
---

# Design System: Fable Flight

## Overview

**Creative North Star: "The Quiet Flight Deck"**

The look Fable Flight already has, flown far better. Dark, near-opaque glass panels sit at the edges of the world like a cockpit after sunset. A single warm yellow lights the one thing to press, the thing that is happening, or the state that matters. Everything else steps back into cool greys so the eye lands where it should in a glance, often while the player is flying.

This is an Operate surface everywhere: menus, the start globe, the career, the HUD. Ambition shows as precision and restraint, not ornament. Every screen earns each word on it, keeps one obvious next action, and leaves the middle of the view to the world. Controls are solid and precise, nearly square, with firm states, closer to switches and annunciators than to web cards.

The system rejects three directions outright: busy sim panels (walls of small text, labels on everything, hint sentences), generic web dashboards (rounded white cards, gradients, coloured stat tiles, an icon on every heading) and neon gamer UI (glowing outlines, angled cut corners, RGB colour, loud motion).

**Key Characteristics:**
* dark glass panels over the live world, near square corners (3px)
* one yellow accent for the primary action, the selection and live state
* Barlow throughout: condensed bold for display, medium and semibold for the interface
* the HUD hugs the screen edges and stacks, it never overlaps itself
* gamepad and keyboard prompts drawn on key caps and pad glyphs, beside the action they trigger

## Colors

A cool navy-black night with one warm light: the accent is the only saturated colour that is not a status.

### Primary
- **Annunciator Yellow** (#F5C518): the primary button (one per screen), the active tab's underline, the chosen card's outline, the heading bug, progress bars, and live values on the HUD. On hover it brightens to **Lamp Yellow** (#FFD648). Text on it is **Soot** (#1A1402), never white.

### Neutral
- **Night** (#080B10): the backdrop of full-screen menus and the loading screen.
- **Surface** (#0F131A): inset areas and fields.
- **Raised** (#1C222C) and **Raised Lit** (#2A323F): plain buttons at rest and on hover.
- **Chosen Slate** (#424E62): a selected choice among equals (a start type, a runway, a filter).
- **Panel Glass** (rgba(10, 13, 19, 0.91)): every menu panel and card, dark enough to read over any scenery.
- **HUD Box** (rgba(11, 13, 18, 0.88)): the HUD's tapes, strip and cards.
- **Hairline** (white at 10%) and **Hairline Lit** (white at 17%): panel edges, dividers, tab rules.
- **Cockpit White** (#E8ECF2): body text. **Dim** (#929CAB): secondary text and inactive tabs. **Faint** (#76808F): disabled and tertiary text, credits.

### Status
- **Nav Blue** (#84B8FF): links and informational marks (the into-wind windsock, map legends).
- **Green Light** (#70D68A): done, passed, earned (checklist ticks, a passed checkride).
- **Warning Coral** (#FF7A5C): failure, crash, money owed, a warning plate.
- **Caution Amber** (#FFC450): HUD cautions (low fuel, flap overspeed) and AP OFF.

### Named Rules
**The One Light Rule.** One filled yellow element per screen: the action the player came for. Everything else that needs yellow uses it as a line, a bar or a number, never a second filled button.

**The Status Means Status Rule.** Green, coral and amber only ever report a state. They are never decoration, never a category colour on a card that is not reporting something. The one category palette is the career's kinds of work, and it lives only in their small tags, tier dots and route lines, never as a fill or a bar.

## Typography

**Display Font:** Barlow Condensed Bold (with the system sans as fallback)
**Body Font:** Barlow (Regular, Medium, SemiBold)
**Label/Mono Font:** IBM Plex Mono Medium, for data only

**Character:** a condensed, upright aviation display face for titles and big numbers, set against Barlow's plain humanist sans for everything the player reads in a hurry.

### Hierarchy
- **Display** (Barlow Condensed 700, 44px screen titles up to 112px on the FLY tile, tight leading): screen titles (CAREER, PILOT, PAUSED), the wordmark, choice card words, the primary button's label in capitals.
- **Headline** (Barlow 600, 28px): page titles inside panels.
- **Title** (Barlow 600, 17px): an item's name in a list or card, the step to do on a tutorial card.
- **Label** (Barlow 500, 15px): buttons, tabs, rows, values, key caps.
- **Body** (Barlow 400, 15px): the rare full sentence, a description, an empty state.
- **Caption** (Barlow 400, 13px): secondary facts under a name (distance, time, skill).
- **Section** (Barlow Condensed 700, 15px, capitals, 1px tracking): a card's title (LOOKING FOR WORK, RECORDS). Not used above another heading.
- **Data** (IBM Plex Mono 500, 13px): coordinates, frequencies and code. Not for counts or labels.

### Named Rules
**The Same Face Rule.** Counts, fractions and progress ("0 / 4", "2 / 6", "0 / 300 XP") use the label face, not the mono. Mono is for data a pilot copies, never for decoration.

**The No Kicker Rule.** No small label above a heading. A heading carries its own weight; the context goes on the same line, dimmed, or nowhere.

## Layout

Menus are full screen: a page head (back button, display title, tabs, the pilot's level and money at the right) over a hairline, then content in one or two columns of cards with 16px between cards. The start globe keeps two floating panels at the left and right edges with the globe between them. Every inset and gap is on the 4, 8, 12, 16, 24 scale: panel insets 24px (16px on the start globe's narrow panels and in-flight cards), card insets 16px by 24px; rows inside 4px to 8px apart; sections 24px apart (tight within a group, generous between). Rows are 32, 40 or 48px tall, never in between.

In flight the middle of the screen belongs to the world. The heading strip hangs top centre; the speed tape sits on the left edge and the altitude tape on the right, centred a little above the middle; the minimap is bottom left; tutorial and job cards are top right; the autopilot panel, alerts and ATC subtitles stack upward from the bottom centre; achievements arrive bottom right, experience bottom left above the map.

Layouts are checked at 1920x1080, at 1536x864 (1080p at 125% scaling) and at 1280x720.

### Named Rules
**The Stack Rule.** Overlays that share an edge stack, measured from what is already there (`flightui::bottom_stack`, `right_top`). No fixed offsets that assume nothing else is showing.

**The Clear Middle Rule.** Nothing but the flight path marker and traffic tags sits in the middle third of the flight view.

## Elevation & Depth

Flat and tonal. Depth comes from the glass panels' opacity over the world and from a hairline edge, not from stacked shadows. Menu panels carry one soft shadow to lift them off the globe; HUD panels carry none. Full-screen choices (fly's two cards) dim everything behind them to near black instead of floating over a busy scene.

### Shadow Vocabulary
- **Panel lift** (offset 0 10px, blur 32px, black at 47%): menu panels only (`menu::panel`).

### Named Rules
**The Hairline Rule.** A panel is defined by its fill and a 1px hairline (white at 17%). Never a coloured side bar, never a glow.

## Shapes

Nearly square: 3px corners on panels, cards, buttons, fields and key caps; square corners on the HUD. Circles are kept for things that are round in a cockpit: the level ring, avatars, status dots, the tick boxes' marks. Selection is shown by fill and outline, not by shape change.

## Components

### Buttons
- **Shape:** nearly square (3px).
- **Primary:** Annunciator Yellow fill, Soot text in the display face in capitals, 46px tall in panels and 50px in the pause menu. One per screen: Take off, Fly from, Resume, Get licence.
- **Hover / Focus:** primary brightens to Lamp Yellow; plain buttons go from Raised to Raised Lit with a lighter hairline. The gamepad's focus draws a yellow ring around the focused control.
- **Plain:** Raised fill, Cockpit White label in the label face, 32px tall.
- **Quiet action:** a dim word that underlines on hover, for what is rarely wanted (Abandon, Reset, Rebuild route).
- **Prompts:** a button that has a key shows it on a key cap at its right (Esc, R, Enter); with a pad it shows the pad's glyph instead.

### Choices and Tabs
- **Choice:** a row or grid of even buttons; the chosen one fills Chosen Slate with an Annunciator Yellow outline.
- **Tabs:** words in the label face (capitals in page heads), Dim at rest and white when active, with a 3px yellow bar under the active one. Q and E (or LB and RB) key caps sit at the two ends and turn the tabs.

### Cards / Containers
- **Corner Style:** 3px.
- **Background:** Panel Glass with a Hairline Lit edge.
- **Shadow Strategy:** none inside a screen (see Elevation).
- **Internal Padding:** 16px by 24px.
- **Content:** a section title or a title, then rows. Empty cards are hidden; a card shows only when it has something to say.

### Inputs / Fields
- **Style:** Surface fill, hairline edge, 3px corners, label face text, a dim placeholder that names what to type (ICAO code, city or airport name).
- **Focus:** a lighter edge; with a pad the yellow focus ring.

### Navigation
- **Page head:** a back button (chevron and the word), the screen title in display, the tab strip, and the status at the right. A hairline below.
- **Flight menu:** the page's name, an icon row for the pages with the current one filled yellow, and a close button. Pointing at (or pad-focusing) an icon shows that page's name in the title, dimmed. Its Flight page is an instrument list: name, value and a dim detail in three aligned columns, hairlines between groups, never tiles with a label over a big number.

### HUD Panels
- **Style:** HUD Box fill, square, no border radius, numbers in Barlow SemiBold, live values white, the bug and cautions in yellow and amber.
- **Tapes:** the speed and altitude tapes carry their tags inside, a head and a foot row (G/S, V/S above; THR, AGL below) on the tape's own fill, never floating over the sky. The readout is one number in one size in a box with a yellow hairline and a notch at the scale, like the heading's; no rolling drum.
- **Cards in flight:** a title and the one thing to do now, a thin progress bar, a quiet abandon link. 300px wide, top right.
- **Autopilot panel:** a mode control panel along the bottom: the AP master, then a column each for HDG, the courses (NAV, APR, GPS), ALT, VS and A/T. Each column is its mode button over the value it aims for. A mode button is green while it flies, outlined in white while armed, dim when off. The value window has minus and plus (held to repeat), turns with the wheel (Shift for big steps) and takes the aircraft's own value on a click. No knobs, no separate annunciation line, one face.
- **Flight menu pages:** never tiles. Columns and rows sit flat on the card, split by hairlines, each part named by a dim label (not spaced capitals) with its value in the 17px value size.

### Notifications
- **Achievements and levels:** a compact card in the bottom right (an icon tile, a dim caption, the name), sliding up over 0.35s, held 5s.
- **Experience:** a translucent line bottom left (+25 XP in yellow, the reason in white), fading after 2.6s.

### Key Caps
- A raised slate face (#3A4454) on a darker foot, white label in Barlow Medium 13.5px, at least 26px wide and 24px tall. The same cap everywhere a key is shown.

## Do's and Don'ts

### Do:
- **Do** keep one filled yellow button per screen and make it the action the player came for.
- **Do** show what a control does with its key or pad glyph beside it, drawn on a cap.
- **Do** stack overlays from what is already on screen (The Stack Rule) and check 1536x864.
- **Do** hide empty cards and show a button only when pressing it works.
- **Do** use the display face for titles and big numbers, the label face for everything interactive.
- **Do** fade between screens and tabs quickly (about 0.26s, eased out) and slide notifications in.

### Don't:
- **Don't** add hint sentences, summary lines or "Lesson:" style prefixes; the control and its label say it.
- **Don't** build busy sim panels: walls of small text, labels on every element, several competing highlights.
- **Don't** drift into a generic web dashboard: rounded white cards, gradients, coloured stat tiles, an icon on every heading.
- **Don't** use neon gamer styling: glowing outlines, angled cut corners, RGB colour, loud motion.
- **Don't** put a small label above a heading, or a coloured bar down one side of a box.
- **Don't** use Unicode arrows or emoji as icons; draw them.
- **Don't** set counts or labels in the mono face.
