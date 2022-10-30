""" _summary_

    Regex static patterns to use in the config file parsers
"""

RE_ATTRIBUTES: str = r"^\[\[#\w+]]"

RE_VALID_LINE_FORMAT: str = r"^\[\[#\w+]]$|^# ?.+$|^.+: ?.*|^(?=\t|\s{4}).+$"

# Pattern to retrieve all lines who are attributes [[#attr]] or properties
BLOCK_PATTERN: str = r"(?s)^[^#|^\n].*?(?=^\[\[#\w+]])|^\[\[#\w+]]\n^\w+: ?.+"
