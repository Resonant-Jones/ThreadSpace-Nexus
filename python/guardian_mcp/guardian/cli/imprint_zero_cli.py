from argparse import Namespace, _SubParsersAction
from guardian.cli.base_command import BaseCommand
import json
from guardian.imprint_zero_onboarding import ImprintZero as ImprintZeroCore

class ImprintZeroCommand(BaseCommand):
    @staticmethod
    def name() -> str:
        return "dump-imprint-zero-prompt"

    @staticmethod
    def help_text() -> str:
        return "Dump the ImprintZero prompt."

    def execute(self, args: Namespace) -> None:
        core = ImprintZeroCore()
        user_prompt = getattr(core, "question_scaffold", "")
        system_prompt = getattr(core, "system_prompt", "")
        if args.json_output:
            prompt_data = {
                "system_prompt": system_prompt,
                "question_scaffold": user_prompt,
            }
            print(json.dumps(prompt_data, indent=2))
        else:
            text = f"--- System Prompt ---\n{system_prompt}\n\n--- Question Scaffold ---\n{user_prompt}"
            print(text)

    @classmethod
    def register(cls, subparsers: _SubParsersAction) -> None:
        parser = subparsers.add_parser(cls.name(), help=cls.help_text())
        parser.add_argument("--json-output", "-j", action="store_true", help="Output in JSON format")
        parser.set_defaults(func=cls.run)
