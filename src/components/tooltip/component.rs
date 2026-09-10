use dioxus::prelude::*;
use dioxus_primitives::tooltip::{self, TooltipContentProps, TooltipProps, TooltipTriggerProps};

// Loaded once from the app root (main.rs) instead of via a nested document::Link here —
// dioxus-desktop doesn't inject document::Link stylesheets declared inside a child
// component's own render into <head>, only ones declared at the App() root.
pub const STYLE_CSS: Asset = asset!("./style.css");

/// Keeps every open tooltip inside the viewport. Call once from `App()`.
///
/// `data-side` is a fixed choice per call site, so a `left` tooltip on a trigger near the
/// left edge renders off-screen — the talent tree's first column does exactly that below
/// ~1000px. No static side is right at every width, so measure the box on open, mirror it
/// to the opposite side if that fits, and nudge it in if neither does.
///
/// Nudging uses the `translate` property, which composes with the CSS
/// `transform: translate*(-50%)` centring instead of overwriting it, and the width is
/// capped against the measured viewport rather than `100vw` — Android's WebView reports a
/// fake ~980px for the latter. `data-shifted` marks a box that moved, so style.css can
/// hide an arrow that no longer points at anything.
pub fn init_positioning() {
    document::eval(
        r#"
        if (!window.__dxTooltip) {
            const PAD = 8;
            const MAX_WIDTH = 400;
            const GAP = 8;
            const place = (el) => {
                if (el.getAttribute('data-state') !== 'open') {
                    return;
                }
                // Undo the previous pass first, or the offsets compound.
                el.style.maxWidth = '';
                el.style.translate = '';
                el.style.left = '';
                el.style.right = '';
                el.style.marginLeft = '';
                el.style.marginRight = '';
                el.removeAttribute('data-shifted');

                const vw = document.documentElement.clientWidth;
                const vh = document.documentElement.clientHeight;
                // A box wider than the screen can't be fitted into it.
                el.style.maxWidth = Math.min(MAX_WIDTH, vw - 2 * PAD) + 'px';

                const side = el.getAttribute('data-side');
                let r = el.getBoundingClientRect();
                const overflowsX = r.left < PAD || r.right > vw - PAD;

                // Mirror to the opposite side when this one doesn't fit and the other
                // does — nudging alone would drop the box on top of its own trigger.
                if (overflowsX && (side === 'left' || side === 'right')) {
                    const anchor = el.parentElement.getBoundingClientRect();
                    if (side === 'left' && anchor.right + GAP + r.width <= vw - PAD) {
                        el.style.right = 'auto';
                        el.style.left = '100%';
                        el.style.marginRight = '0';
                        el.style.marginLeft = GAP + 'px';
                    } else if (side === 'right' && anchor.left - GAP - r.width >= PAD) {
                        el.style.left = 'auto';
                        el.style.right = '100%';
                        el.style.marginLeft = '0';
                        el.style.marginRight = GAP + 'px';
                    }
                    el.setAttribute('data-shifted', 'true');
                    r = el.getBoundingClientRect();
                }

                // Whatever side it ended up on, keep it on screen.
                let dx = 0;
                if (r.left < PAD) {
                    dx = PAD - r.left;
                } else if (r.right > vw - PAD) {
                    dx = vw - PAD - r.right;
                }
                let dy = 0;
                if (r.top < PAD) {
                    dy = PAD - r.top;
                } else if (r.bottom > vh - PAD) {
                    // Never push the top off-screen to save the bottom.
                    dy = Math.max(vh - PAD - r.bottom, PAD - r.top);
                }
                if (dx || dy) {
                    el.style.translate = `${dx}px ${dy}px`;
                    el.setAttribute('data-shifted', 'true');
                }
            };
            const placeOpen = () =>
                document.querySelectorAll('.tooltip-content[data-state="open"]').forEach(place);

            // The primitive renders the content only while open (`if render()` in
            // dioxus-primitives' tooltip.rs), so opening *inserts* the element rather than
            // flipping an attribute on a hidden one — this has to watch childList, not just
            // data-state. Attributes are still watched for the close animation, which
            // re-renders the node in place.
            const consider = (node) => {
                if (node.nodeType !== 1) {
                    return;
                }
                if (node.classList.contains('tooltip-content')) {
                    place(node);
                }
                node.querySelectorAll('.tooltip-content').forEach(place);
            };
            new MutationObserver((records) => {
                for (const rec of records) {
                    if (rec.type === 'childList') {
                        rec.addedNodes.forEach(consider);
                    } else {
                        consider(rec.target);
                    }
                }
            }).observe(document, {
                subtree: true,
                childList: true,
                attributes: true,
                attributeFilter: ['data-state'],
            });
            window.addEventListener('resize', placeOpen);
            window.__dxTooltip = { placeOpen };
            console.debug('[dxTooltip] viewport clamping active');
        }
        "#,
    );
}

#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    rsx! {
        tooltip::Tooltip {
            class: "tooltip",
            disabled: props.disabled,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    rsx! {
        tooltip::TooltipTrigger {
            class: "tooltip-trigger",
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn TooltipContent(props: TooltipContentProps) -> Element {
    rsx! {
        tooltip::TooltipContent {
            class: "tooltip-content",
            id: props.id,
            side: props.side,
            align: props.align,
            attributes: props.attributes,
            {props.children}
        }
    }
}
