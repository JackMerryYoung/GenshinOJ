import { Button, Popover, PopoverTrigger, PopoverSurface } from "@fluentui/react-components";
import { EmojiRegular } from "@fluentui/react-icons";
import { useTranslation } from "react-i18next";

import { EMOJI_SHORTCUTS } from "../EmojiShortcuts.ts";

// Shared emoji picker — same `\ll`/`\zan` shortcut set chat uses. `onPick` receives the chosen
// emoji (callers typically append it to their text field).
export default function EmojiPicker({ onPick }: { onPick: (emoji: string) => void }) {
    const { t } = useTranslation("emojiPicker");

    return <Popover>
        <PopoverTrigger disableButtonEnhancement>
            <Button size="small" appearance="subtle" icon={<EmojiRegular />} title={t("insertEmoji")} />
        </PopoverTrigger>
        <PopoverSurface>
            <div style={{ display: "flex", flexWrap: "wrap", maxWidth: "240px" }}>
                {
                    Object.entries(EMOJI_SHORTCUTS).map(([shortcut, emoji]) => (
                        <Button
                            key={shortcut}
                            appearance="subtle"
                            title={shortcut}
                            onClick={() => onPick(emoji)}
                            style={{ fontSize: "1.2em", minWidth: "2em" }}>
                            {emoji}
                        </Button>
                    ))
                }
            </div>
        </PopoverSurface>
    </Popover>;
}
