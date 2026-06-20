// QQ-style text-shortcut emoticons. Typing one of these in a chat message and sending it
// replaces the shortcut with the corresponding emoji, the same way QQ replaces `/ll` etc.
export const EMOJI_SHORTCUTS: Record<string, string> = {
    "\\ll": "😢", // tear dropping / 委屈
    "\\xx": "😄", // laughing
    "\\wx": "🙂", // smile
    "\\dk": "😮", // shocked / 吃惊
    "\\nu": "😠", // angry
    "\\ku": "😭", // crying
    "\\se": "😳", // blushing
    "\\fn": "😡", // furious
    "\\tu": "😝", // tongue out
    "\\yun": "😵", // dizzy
    "\\shui": "😴", // sleepy
    "\\ai": "😔", // sigh / sad
    "\\heng": "😤", // huffing
    "\\zan": "👍", // thumbs up
    "\\ai_xin": "❤️", // heart
    "\\qiang": "💪", // strong
    "\\zhutou": "🐷", // pig head (笑哭/猪头)
    "\\bs": "🤮", // disgusted / 鄙视
    "\\hua": "🌹", // rose
    "\\dan": "🥚", // egg
};

// Sorted longest-shortcut-first so e.g. `\\ai_xin` isn't shadowed by a shorter overlapping key.
const SHORTCUT_KEYS = Object.keys(EMOJI_SHORTCUTS).sort((a, b) => b.length - a.length);

export function replaceEmojiShortcuts(text: string): string {
    let result = text;
    for (const shortcut of SHORTCUT_KEYS) {
        result = result.split(shortcut).join(EMOJI_SHORTCUTS[shortcut]);
    }
    return result;
}
