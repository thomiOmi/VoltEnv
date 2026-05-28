# Design System: Nuxt UI
**Project ID:** nuxt/ui — https://ui.nuxt.com

## 1. Visual Theme & Atmosphere

Nuxt UI embodies a **confident, developer-centric precision** that fuses the clarity of modern SaaS interfaces with the quiet authority of open-source craftsmanship. The interface feels **structured and purposeful**, built on the philosophy that great tooling should be invisible — the UI gets out of the way and lets the product shine. Every element earns its presence.

The overall mood is **clean and trustworthy with a pulse of energy** — not cold or sterile, but crisp and efficient. The design language communicates professionalism without austerity: it invites exploration while rewarding focus. The Vivid Spring Green accent cuts through the neutral canvas like a well-placed highlight, signaling action and momentum without overwhelming the senses. The atmosphere is reminiscent of a well-lit modern workspace — ordered, capable, and quietly inspiring.

**Key Characteristics:**
- Disciplined whitespace that organizes information into scannable, breathable zones
- Neutral-dominant palette with a single energetic green accent for interactive moments
- Flat, near-shadowless surfaces that feel modern and performance-oriented
- Consistent, systematic spacing that creates rhythm and visual predictability
- Components that feel polished but never decorative — function drives form
- Dual-mode fluency: light mode feels like a crisp document; dark mode feels like a focused cockpit

## 2. Color Palette & Roles

### Primary Foundation
- **Pristine White** (`#FFFFFF`) - Primary background in light mode. The pure, distraction-free canvas upon which all content rests. Communicates clarity and openness.
- **Ghost Surface White** (`#F8FAFC`) - Secondary surface color for muted backgrounds, sidebar fills, table alternation rows, and input backgrounds. Barely perceptible warmth that separates layers without introducing noise.
- **Deep Midnight Slate** (`#0F172A`) - Primary background in dark mode. A deep, near-black navy that creates an immersive yet comfortable dark canvas — not harsh black, but rich and enveloping.

### Accent & Interactive
- **Vivid Spring Green** (`#00DC82`) - The system's heartbeat and primary brand color. Used exclusively for primary CTAs, active navigation states, focus rings, progress indicators, and key interactive highlights. Electrifying but not aggressive — it draws the eye decisively without shouting.
- **Emerald Hover** (`#00C16A`) - The pressed and hover state of the primary green. A step deeper, providing tactile feedback that confirms interaction with satisfying subtlety.

### Semantic / Functional Colors
- **Ocean Informational Blue** (`#3B82F6`) - Calm and analytical. Applied to secondary actions, info alerts, helper tooltips, and links requiring neutral guidance. Signals "more context available" without urgency.
- **Amber Caution** (`#EAB308`) - Warm and deliberate. Reserved strictly for warnings, pending states, and items requiring careful user attention. Never used decoratively.
- **Crimson Alert Red** (`#EF4444`) - Sharp and unambiguous. Exclusively marks error states, validation failures, destructive action confirmations, and critical system feedback. Used sparingly for maximum signal impact.
- **Soft Jade Confirmation** (`#22C55E`) - Gentle affirmation. Marks successful operations, completed states, and positive confirmations. Intentionally lighter in tone than the primary green to prevent confusion.

### Typography & Text Hierarchy
- **Storm Slate** (`#334155`) - Primary body text color in light mode. Strong and readable without the harshness of pure black — sophisticated and refined.
- **Fog Slate** (`#64748B`) - Secondary and muted text. Supporting descriptions, placeholders, helper text, and metadata. Recedes gracefully, supporting without competing.
- **Pale Mist Border** (`#E2E8F0`) - Hairline borders, dividers, and structural separators in light mode. So subtle it feels like negative space rather than an element.
- **Elevated Night** (`#1E293B`) - Card and panel backgrounds in dark mode. Lifted slightly above the base midnight, creating quiet depth and surface hierarchy.

## 3. Typography Rules

**Primary Font Family:** Public Sans  
**Character:** A utilitarian humanist sans-serif designed for government and civic interfaces — highly legible, neutral in personality, yet with quiet warmth in its letterforms. Feels professional without being corporate.

**Secondary Font Family (Code):** JetBrains Mono  
**Character:** Sharp, purposeful, and unambiguously technical. Applied to code blocks, keyboard shortcuts, and inline code references. Every character is distinct for maximum legibility in dense code contexts.

### Hierarchy & Weights
- **Display Headlines (H1):** Bold weight (700-800), tight letter-spacing (-0.02em for gravitas), 2.5-3.5rem size. Reserved for hero sections, page titles, and landmark headings.
- **Section Headers (H2):** Semibold weight (600), neutral letter-spacing (0em), 1.75-2.25rem size. Structures content zones with authority and calm.
- **Subsection Headers (H3):** Semibold weight (600), 1.25-1.5rem size. Component labels, feature names, and card titles.
- **Body Text:** Regular weight (400), generous line-height (1.65-1.7), 1rem size. Optimized for long-form documentation and instruction readability.
- **Small Text / Meta:** Regular weight (400), 0.875rem size, slightly tighter line-height (1.5). Prices, timestamps, badges, and supporting metadata — present but visually recessive.
- **CTA Labels:** Medium weight (500), subtle letter-spacing (0.01em), 0.875-1rem size. Confident without aggression.
- **Code / Monospace:** Regular weight (400), JetBrains Mono, 0.875rem size. Maximum character distinction for code legibility.

### Spacing Principles
- Display headers use mildly compressed letter-spacing for punchy authority
- Body text maintains generous line-height for effortless reading in documentation-heavy contexts
- Consistent vertical rhythm using 4px base unit multiples throughout
- Clear visual distance (1.5-2rem) between typographic groups to enforce hierarchy

## 4. Component Stylings

### Buttons
- **Shape:** Softly rounded corners (6px/0.375rem radius) — modern and approachable without veering playful; a decisive, professional silhouette
- **Primary CTA:** Vivid Spring Green (`#00DC82`) fill with deep near-black text, comfortable padding (0.5-0.625rem vertical, 1rem-1.25rem horizontal)
- **Hover State:** Shifts to Emerald Hover (`#00C16A`) with a smooth 150ms ease-in-out transition — quick and responsive, matching developer expectations
- **Focus State:** Vivid Spring Green outer ring (2px offset focus ring) for clear keyboard navigation accessibility
- **Secondary / Outline:** Transparent fill with Pale Mist Border border; hover introduces a Ghost Surface White fill wash
- **Ghost / Link Button:** Text-only, no border or fill; hover shifts text to Vivid Spring Green
- **Destructive:** Crisp Crimson Alert Red fill, reserved strictly for irreversible actions

### Cards & Containers
- **Corner Style:** Softly rounded corners (8px/0.5rem radius) — consistent with buttons, maintaining system-wide visual cohesion
- **Background (Light):** Pristine White (`#FFFFFF`) surface sitting on a Ghost Surface White page
- **Background (Dark):** Elevated Night (`#1E293B`) panel raised above the Deep Midnight Slate base
- **Border:** Single hairline (1px) in Pale Mist Border (`#E2E8F0`) in light mode; near-invisible dark border (`#334155`) in dark mode
- **Shadow Strategy:** Effectively flat by default — no shadow on resting state. Modals and dropdowns gain a whisper-soft diffused shadow (`0 4px 16px rgba(0,0,0,0.08)`) to communicate elevation
- **Internal Padding:** Generous 1.5rem (24px) creating comfortable, breathable content zones

### Navigation & Sidebar
- **Top Navigation:** Clean, horizontal, generous item spacing (1.5-2rem gaps). Items use Storm Slate text, transitioning smoothly to Vivid Spring Green on hover or active state
- **Active Indicator:** A subtle Vivid Spring Green underline or left-border accent depending on orientation (horizontal vs. vertical)
- **Sidebar:** Fixed-width, collapsible panel in Ghost Surface White (light) or Elevated Night (dark). Items stack vertically with consistent 0.5rem vertical padding per item
- **Mobile Collapse:** Full-width overlay drawer, sliding in from the left, with smooth 250ms ease transform

### Inputs & Forms
- **Stroke Style:** Refined 1px Pale Mist Border border on a Pristine White background (light) / Elevated Night background (dark)
- **Focus State:** Border instantly updates to Vivid Spring Green with a soft green outer glow ring — unmistakable and satisfying
- **Error State:** Border and ring shift to Crimson Alert Red with a helper error message in the same red below the field
- **Corner Style:** Matching the button and card roundness (6-8px/0.375-0.5rem) for system-wide consistency
- **Padding:** Comfortable 0.625-0.75rem vertical, 0.875rem horizontal for accessible touch targets
- **Placeholder Text:** Fog Slate (`#64748B`) — readable but clearly recessive, never competing with entered content

### Badges & Tags
- **Shape:** Fully pill-shaped (fully rounded, stadium silhouette) for compact, friendly status indicators
- **Color Strategy:** Soft tinted fill (10-15% opacity semantic color wash) with full-strength semantic color text — never harsh, always legible
- **Size:** Compact, 0.75rem text with tight horizontal padding (0.5-0.75rem). Meant to inform at a glance, not dominate the layout

### Modals & Overlays
- **Backdrop:** Semi-transparent dark scrim (`rgba(0,0,0,0.5)`) with a quick 200ms fade-in
- **Panel:** Pristine White (light) / Deep Midnight Slate (dark) with generously rounded corners (12px/0.75rem) — noticeably more curved than cards for architectural presence
- **Shadow:** Prominent but diffused (`0 20px 60px rgba(0,0,0,0.15)`) grounding the panel with clear elevation
- **Animation:** Slides in from slightly below with a subtle scale (0.95 → 1.0), feeling physical and responsive

### Toasts & Notifications
- **Position:** Bottom-right corner stack, appearing with a smooth slide-up entrance
- **Style:** Compact, pill-adjacent rounded corners, tinted semantic left-border accent for immediate color-coded status recognition
- **Duration:** Auto-dismiss after 5 seconds with a subtle progress indicator; hover pauses the timer

### Tables
- **Header Row:** Ghost Surface White background with Storm Slate medium-weight text, bottom-bordered with Pale Mist Border
- **Data Rows:** Pristine White default; alternate row tinting optional (Ghost Surface White) for dense data sets
- **Row Hover:** Soft Ghost Surface White wash transitioning smoothly on 150ms
- **Borders:** Horizontal row dividers only in Pale Mist Border — vertical borders are omitted for a cleaner, less caged feel

## 5. Layout Principles

### Grid & Structure
- **Max Content Width:** 80rem (1280px) centered for optimal readability; dashboard layouts extend to full viewport width
- **Grid System:** Responsive 12-column grid with consistent fluid gutters (16px mobile, 24px tablet, 32px desktop)
- **Breakpoints:**
  - Mobile: `< 640px`
  - Small: `640px` (sm)
  - Medium: `768px` (md)
  - Large: `1024px` (lg)
  - Extra Large: `1280px` (xl)
  - 2XL: `1536px` (2xl)

### Whitespace Strategy
- **Base Unit:** 4px micro-unit, scaling in multiples (4, 8, 12, 16, 24, 32, 48, 64, 96px)
- **Component Internal Spacing:** 8-16px (tight, grouped relationships)
- **Between Components:** 24-32px (breathing room within sections)
- **Between Sections:** 48-96px (dramatic section separation, reinforcing content independence)
- **Page Edge Padding:** 16px mobile, 24px tablet, 32-48px desktop

### Alignment & Visual Flow
- **Default Alignment:** Left-anchored for all content — body text, labels, navigation, form fields
- **Centered Contexts:** Hero headlines, empty states, modal content, and standalone CTA blocks only
- **Reading Direction:** Top-to-bottom, left-to-right with deliberate focal points guiding eye movement
- **Dashboard Layouts:** Fixed-left sidebar + fluid main content area; content stacks responsively as viewport narrows

### Responsive Behavior
- **Mobile-First Foundation:** Core functionality designed for smallest screens, enhanced progressively
- **Touch Targets:** Minimum 44x44px for all interactive elements (WCAG compliant)
- **Sidebar Collapse:** Desktop sidebars collapse to icon-only or full off-canvas drawer on mobile
- **Typography Scaling:** Fluid type scaling reduces heading sizes gracefully at smaller breakpoints

---

## 6. Design System Notes for AI Generation

When creating new screens for this project, reference these specific prompt instructions:

### Language to Use
- **Atmosphere:** "Clean, developer-focused precision with a pulse of spring green energy"
- **Button Shapes:** "Softly rounded corners" (not `rounded-md` or "6px")
- **Shadows:** "Effectively flat surfaces with whisper-soft diffused shadows reserved for elevated overlays"
- **Spacing:** "Generous, rhythmic breathing room built on a 4px base grid"
- **Dark Mode:** "Deep midnight cockpit feel — immersive without being oppressive"

### Color References
Always pair descriptive names with hex codes:
- Primary Action: "Vivid Spring Green (`#00DC82`)"
- Page Background (Light): "Pristine White (`#FFFFFF`) on Ghost Surface White (`#F8FAFC`)"
- Page Background (Dark): "Deep Midnight Slate (`#0F172A`) with Elevated Night (`#1E293B`) panels"
- Body Text: "Storm Slate (`#334155`)"
- Muted / Helper Text: "Fog Slate (`#64748B`)"
- Borders: "Pale Mist Border (`#E2E8F0`)"

### Component Prompts
- "Create a primary button with softly rounded corners in Vivid Spring Green (`#00DC82`) with comfortable padding and a smooth hover transition to Emerald Hover (`#00C16A`)"
- "Design a data card with 8px rounded corners, a hairline Pale Mist Border border, flat resting surface, and whisper-soft shadow on hover"
- "Build a form input with a refined Pale Mist Border stroke that transitions to a Vivid Spring Green focus ring on interaction"
- "Add a sidebar navigation with Ghost Surface White background, left-aligned items, and Vivid Spring Green active-state left-border accent"
- "Create a modal with generously rounded 12px corners, a deep scrim backdrop, and a subtle slide-up scale entrance animation"

### Incremental Iteration
When refining existing screens:
1. Focus on **ONE component at a time** (e.g., "Update the data table header styling")
2. Be specific about what changes (e.g., "Increase card internal padding from 1rem to 1.5rem and add a hover shadow")
3. Reference this design system language consistently across all prompts
4. Always specify both light and dark mode behavior when modifying surface colors