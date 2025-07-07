import sys

# The actual ASCII Escape character (0x1b)
# This is the non-printable character that terminals understand for ANSI codes.
ESC_CHAR = "\x1b"

def convert_literal_ansi_to_actual(text_with_literal_escapes):
    converted_text = text_with_literal_escapes.replace(r'\e', ESC_CHAR)

    return converted_text


if __name__ == "__main__":
    input_text = sys.stdin.read()

    output_text = convert_literal_ansi_to_actual(input_text)

    sys.stdout.write(output_text)
