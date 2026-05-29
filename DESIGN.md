# Design System: Nuxt UI
**Project ID:** nuxt/ui — https://ui.nuxt.com

## 1. Visual Theme & Atmosphere

Nuxt UI embodies a **confident, developer-centric precision** that fuses the clarity of modern SaaS interfaces with the quiet authority of open-source craftsmanship. The interface feels **structured and purposeful**, built on the philosophy that great tooling should be invisible — the UI gets out of the way and lets the product shine. Every element earns its presence.

The overall mood is **clean and energetic with a warm streak of amber boldness** — not cold or sterile, but sharp and driven. The design language communicates capability and modernity: the Golden Amber accent cuts through the pure neutral canvas like a beam of sunlight, signaling action and confidence without overwhelming the senses. The atmosphere evokes a well-lit, high-contrast workspace — structured, decisive, and built for people who move fast.

**Key Characteristics:**
- Disciplined whitespace that organizes information into scannable, breathable zones
- True neutral palette — no blue or warm tint bias — with a bold golden amber accent for all interactive moments
- Flat, near-shadowless surfaces that feel modern and performance-oriented
- Consistent, systematic spacing that creates rhythm and visual predictability
- Components that feel polished but never decorative — function drives form
- Dual-mode fluency: light mode feels razor-crisp; dark mode feels like a focused, high-contrast cockpit

## 2. Color Palette & Roles

### Primary Foundation
- **Pure White** (`#FFFFFF`) – Primary background in light mode. The ultimate neutral canvas — no warmth bias, no cool tint. Absolute clarity.
- **Near-White Surface** (`#FAFAFA`) – Secondary surface color for muted backgrounds, sidebar fills, table rows, and subtle content layering. Barely distinguishable from white, creating the softest possible depth.
- **True Black Night** (`#171717`) – Primary background in dark mode. A deep, pure near-black with no blue or warm tint — stark, focused, and high-contrast.

### Accent & Interactive
- **Golden Amber** (`#EAB308`) – The system's defining energy. Used for primary CTAs, active navigation states, focus rings, selected states, and progress indicators. Bold and warm, it creates unmistakable visual anchors against the neutral foundation without feeling garish.
- **Deep Harvest Gold** (`#CA8A04`) – The hover and pressed state for primary actions. A richer, more saturated amber that confirms interaction with satisfying warmth and depth.

### Semantic / Functional Colors
- **Ocean Informational Blue** (`#3B82F6`) – Calm and analytical. Applied to secondary actions, info alerts, helper tooltips, and neutral guidance links.
- **Golden Warning** (`#EAB308`) – Note: shares the primary accent. In warning contexts, it is used with appropriate supporting iconography and label copy to differentiate from interactive elements.
- **Crimson Alert Red** (`#EF4444`) – Sharp and unambiguous. Exclusively marks error states, validation failures, destructive confirmations, and critical system feedback.
- **Soft Jade Confirmation** (`#22C55E`) – Gentle affirmation. Marks successful operations, completed states, and positive confirmations.

### Typography & Text Hierarchy
- **Near-Black Ink** (`#171717`) – Headline and primary text in light mode. Pure, high-contrast, and uncompromising — the maximum readable signal against a white canvas.
- **Mid Neutral Gray** (`#737373`) – Secondary and muted text. Supporting descriptions, placeholders, helper text, and metadata. Neutral through and through — no blue, no warm cast.
- **Soft Neutral Border** (`#E5E5E5`) – Hairline borders, dividers, and structural separators in light mode. True neutral gray — clean, invisible-feeling separation.
- **Lifted Dark Surface** (`#262626`) – Card and panel backgrounds in dark mode. A step above the True Black Night base, creating quiet, pure neutral depth.

## 3. Typography Rules

**Primary Font Family:** Geist  
**Character:** Vercel's purpose-built typeface for developer interfaces — a modern geometric sans-serif with exceptional screen legibility. Precise, unornamented, and highly technical in spirit, yet with subtle humanist warmth in its curves. Designed specifically for both UI text and code-adjacent contexts, making it uniquely suited for developer tooling.

**Secondary Font Family (Code):** Geist Mono  
**Character:** The monospace companion to Geist — perfectly harmonized with the primary typeface. Applied to code blocks, keyboard shortcuts, terminal output, and inline code references. Shares Geist's geometric DNA, making the transition between prose and code feel seamless.

### Hierarchy & Weights
- **Display Headlines (H1):** Bold weight (700), tight letter-spacing (-0.025em for technical authority), 2.5–3.5rem size. Reserved for hero sections, page titles, and landmark headings.
- **Section Headers (H2):** Semibold weight (600), neutral letter-spacing (-0.01em), 1.75–2.25rem size. Structures content zones with clean, sharp authority.
- **Subsection Headers (H3):** Semibold weight (600), 1.25–1.5rem size. Component labels, feature names, and card titles.
- **Body Text:** Regular weight (400), comfortable line-height (1.6–1.65), 1rem size. Geist's clarity at body size makes documentation and instructions effortless to consume.
- **Small Text / Meta:** Regular weight (400), 0.875rem size, tighter line-height (1.5). Timestamps, badges, and supporting metadata — present but recessive.
- **CTA Labels:** Medium weight (500), neutral letter-spacing (0em), 0.875–1rem size. Geist's geometric forms make button labels feel crisp and decisive.
- **Code / Monospace:** Regular weight (400), Geist Mono, 0.875rem size. Maximum character distinction; harmonizes visually with the primary typeface.

### Spacing Principles
- Display headers use mildly compressed letter-spacing for sharp, technical authority
- Body text maintains comfortable line-height for documentation-heavy reading contexts
- Consistent vertical rhythm using 4px base unit multiples throughout
- Clear visual distance (1.5–2rem) between typographic groups to enforce hierarchy

## 4. Component Stylings

### Buttons
- **Shape:** Softly rounded corners (6px/0.375rem radius) — modern and sharp without veering playful; clean and purposeful
- **Primary CTA:** Golden Amber (`#EAB308`) fill with near-black text (`#171717`), comfortable padding (0.5–0.625rem vertical, 1rem–1.25rem horizontal). The dark text on amber creates a bold, high-contrast pairing.
- **Hover State:** Shifts to Deep Harvest Gold (`#CA8A04`) with a smooth 150ms ease-in-out transition — immediate and responsive
- **Focus State:** Golden Amber outer ring (2px offset focus ring) for clear keyboard navigation accessibility
- **Secondary / Outline:** Transparent fill with Soft Neutral Border (`#E5E5E5`) border; hover introduces a Near-White Surface wash
- **Ghost / Link Button:** Text-only, no border or fill; hover shifts text to Golden Amber (`#EAB308`)
- **Destructive:** Crimson Alert Red fill, reserved strictly for irreversible actions

### Cards & Containers
- **Corner Style:** Softly rounded corners (8px/0.5rem radius) — consistent with buttons for system-wide visual cohesion
- **Background (Light):** Pure White (`#FFFFFF`) surface sitting on a Near-White Surface page
- **Background (Dark):** Lifted Dark Surface (`#262626`) panel raised above the True Black Night base
- **Border:** Single hairline (1px) in Soft Neutral Border (`#E5E5E5`) in light mode; near-invisible dark border (`#404040`) in dark mode
- **Shadow Strategy:** Effectively flat by default. Modals and dropdowns gain a whisper-soft diffused shadow (`0 4px 16px rgba(0,0,0,0.08)`) to communicate elevation
- **Internal Padding:** Generous 1.5rem (24px) creating comfortable, breathable content zones

### Navigation & Sidebar
- **Top Navigation:** Clean, horizontal, generous item spacing (1.5–2rem gaps). Items use Near-Black Ink text, transitioning to Golden Amber on hover or active state
- **Active Indicator:** A precise Golden Amber underline or left-border accent depending on orientation (horizontal vs. vertical)
- **Sidebar:** Fixed-width, collapsible panel in Near-White Surface (light) or Lifted Dark Surface (dark). Vertically stacked items with consistent 0.5rem vertical padding
- **Mobile Collapse:** Full-width overlay drawer sliding from the left with smooth 250ms ease transform

### Inputs & Forms
- **Stroke Style:** Refined 1px Soft Neutral Border (`#E5E5E5`) on Pure White (light) / Lifted Dark Surface (dark) background
- **Focus State:** Border instantly updates to Golden Amber (`#EAB308`) with a soft amber outer glow ring — unmistakable and warm
- **Error State:** Border and ring shift to Crimson Alert Red with helper error message below the field
- **Corner Style:** Matching the button roundness (6–8px/0.375–0.5rem) for system-wide consistency
- **Padding:** Comfortable 0.625–0.75rem vertical, 0.875rem horizontal for accessible touch targets
- **Placeholder Text:** Mid Neutral Gray (`#737373`) — purely neutral, readable but clearly recessive

### Badges & Tags
- **Shape:** Fully pill-shaped (fully rounded, stadium silhouette) for compact, friendly status indicators
- **Color Strategy:** Soft tinted fill (10–15% opacity semantic color wash) with full-strength semantic color text
- **Size:** Compact, 0.75rem text with tight horizontal padding (0.5–0.75rem)

### Modals & Overlays
- **Backdrop:** Semi-transparent pure neutral scrim (`rgba(0,0,0,0.5)`) with a quick 200ms fade-in
- **Panel:** Pure White (light) / True Black Night (dark) with generously rounded corners (12px/0.75rem)
- **Shadow:** Prominent but diffused (`0 20px 60px rgba(0,0,0,0.15)`) grounding the panel with clear elevation
- **Animation:** Slides in from slightly below with a subtle scale (0.95 → 1.0)

### Toasts & Notifications
- **Position:** Bottom-right corner stack, appearing with a smooth slide-up entrance
- **Style:** Compact, pill-adjacent rounded corners, left-border semantic color accent for instant status recognition
- **Duration:** Auto-dismiss after 5 seconds; hover pauses the timer

### Tables
- **Header Row:** Near-White Surface background with Near-Black Ink medium-weight Geist text, bottom-bordered with Soft Neutral Border
- **Data Rows:** Pure White default with optional Near-White Surface alternating tint for dense data sets
- **Row Hover:** Soft Near-White Surface wash on 150ms transition
- **Borders:** Horizontal row dividers only — vertical borders omitted for a cleaner, uncaged feel

## 5. Layout Principles

### Grid & Structure
- **Max Content Width:** 80rem (1280px) centered; dashboard layouts extend to full viewport width
- **Grid System:** Responsive 12-column grid with fluid gutters (16px mobile, 24px tablet, 32px desktop)
- **Breakpoints:**
  - Mobile: `< 640px`
  - Small: `640px` (sm)
  - Medium: `768px` (md)
  - Large: `1024px` (lg)
  - Extra Large: `1280px` (xl)
  - 2XL: `1536px` (2xl)

### Whitespace Strategy
- **Base Unit:** 4px micro-unit, scaling in multiples (4, 8, 12, 16, 24, 32, 48, 64, 96px)
- **Component Internal Spacing:** 8–16px (tight, grouped relationships)
- **Between Components:** 24–32px (breathing room within sections)
- **Between Sections:** 48–96px (dramatic section separation)
- **Page Edge Padding:** 16px mobile, 24px tablet, 32–48px desktop

### Alignment & Visual Flow
- **Default Alignment:** Left-anchored for all content — body text, labels, navigation, form fields
- **Centered Contexts:** Hero headlines, empty states, modal content, and standalone CTA blocks only
- **Reading Direction:** Top-to-bottom, left-to-right with deliberate focal points guiding eye movement
- **Dashboard Layouts:** Fixed-left sidebar + fluid main content area

### Responsive Behavior
- **Mobile-First Foundation:** Core functionality designed for smallest screens, enhanced progressively
- **Touch Targets:** Minimum 44×44px for all interactive elements (WCAG compliant)
- **Typography Scaling:** Fluid type scaling reduces heading sizes gracefully at smaller breakpoints

---

## 6. Design System Notes for AI Generation

### Language to Use
- **Atmosphere:** "Sharp, developer-focused precision with a bold golden amber pulse of energy"
- **Button Shapes:** "Softly rounded corners" (not `rounded-md` or "6px")
- **Shadows:** "Effectively flat surfaces with whisper-soft diffused shadows reserved for elevated overlays"
- **Spacing:** "Disciplined, rhythmic breathing room built on a 4px base grid"
- **Dark Mode:** "Pure neutral midnight — no blue or warm tint, maximum contrast cockpit feel"

### Color References
Always pair descriptive names with hex codes:
- Primary Action: "Golden Amber (`#EAB308`)"
- Primary Hover: "Deep Harvest Gold (`#CA8A04`)"
- Page Background (Light): "Pure White (`#FFFFFF`) on Near-White Surface (`#FAFAFA`)"
- Page Background (Dark): "True Black Night (`#171717`) with Lifted Dark Surface (`#262626`) panels"
- Body Text: "Near-Black Ink (`#171717`)"
- Muted / Helper Text: "Mid Neutral Gray (`#737373`)"
- Borders: "Soft Neutral Border (`#E5E5E5`)"

### Component Prompts
- "Create a primary button with softly rounded corners in Golden Amber (`#EAB308`) with near-black text and a smooth hover transition to Deep Harvest Gold (`#CA8A04`)"
- "Design a data card using Geist font with 8px rounded corners, a hairline Soft Neutral Border, flat resting surface, and whisper-soft shadow on hover"
- "Build a form input with a refined Soft Neutral Border stroke that transitions to a Golden Amber focus ring on interaction"
- "Add a sidebar navigation with Near-White Surface background, left-aligned Geist text, and a Golden Amber active-state left-border accent"
- "Create a modal with generously rounded 12px corners, a pure neutral scrim backdrop, and a subtle slide-up scale entrance animation"

### Incremental Iteration
When refining existing screens:
1. Focus on **ONE component at a time** (e.g., "Update the form input focus states")
2. Be specific about what changes (e.g., "Replace the green focus ring with Golden Amber (`#EAB308`) across all inputs")
3. Reference Geist typeface explicitly in all typography-related prompts
4. Always specify both light and dark mode behavior when modifying surface colors