RE_ATTRIBUTES: str = r"^\[\[#\w+]]"

RE_VALID_LINE_FORMAT: str = r"^\[\[#\w+]]$|^# ?.+$|^.+: ?\S+"

# Pattern to retrieve all lines who are attributes [[#attr]] or properties
VALID_LINE_PATTERN: str = r"^\[\[#\w+]]$|^\w+: ?.+"
