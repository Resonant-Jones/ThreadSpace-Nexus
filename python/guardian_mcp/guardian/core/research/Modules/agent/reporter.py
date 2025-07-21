import json
import secrets
import string
import time
from typing import Optional

from ..model import Model
from ..prompt.reporter import report_plan, report_prompt, report_task
from .agent import Agent


class Reporter(Agent):
    def __init__(self, model: Model):
        self.model: Model = model
        self.todo: list[dict] = []
        self.db: Optional[list[dict]] = None

        self.description = "generateing report"

        self.source: dict[str, dict] = {}
        self.name = "reporter"

        self.length = 4  # the length of uuid

    def set_name(self, name):
        self.name = name

    async def run(self, query: str, data=None):
        """
        based on query and data to write a response
        Maybe we plan what to write and write a report style ?
        """
        self.db = self.data_handler(data=data)
        short_summary = self.data_handler(data)
        print("short summary: ")
        print(short_summary)

        res = await self._planner(query=query, db=short_summary)

        print(f"res{res}")
        time.sleep(3)  # FIXME: Replace with proper async wait or callback system
        # problem it is not yet response and then it return and the problem is it can't extract correct res afterward
        tasks = self._extract_response(res)
        tasks = json.loads(tasks)
        print(tasks)

        r = self._task_handler(tasks)
        print(r)

        return {"agent": "TERMINATE", "data": r, "task": ""}

    def data_handler(self, data):
        if not isinstance(data, list):
            print("error handling data")
            return

        short_summaries = []
        alphabet = string.ascii_letters + string.digits

        # Store processed items with IDs for future referencing
        processed_data = []

        for d in data:
            rand_id = "".join(secrets.choice(alphabet) for _ in range(self.length))

            col = {
                "id": rand_id,
                "url": d.get("url", ""),
                "title": d.get("title", ""),
                "summary": d.get("summary", ""),
                "brief_summary": d.get("brief_summary", ""),
                "keywords": d.get("keywords", []),
            }
            processed_data.append(col)

            if col["summary"] != "":
                short_summaries.append({"id": rand_id, "short_summary": col["summary"]})
        self.source = processed_data
        return short_summaries

    # Suppose you got a short summary id and want to get the long summary
    def get_source(self, summary_ids: list[int]) -> list[object]:
        sources = []
        for item in self.source:
            if item["id"] in summary_ids:
                sources.append(item)
        return sources

    def get_recv_format(self):
        return {"query": "str", "data": "list"}

    def get_send_format(self):
        return {"agent": "str", "data": "str", "task": "str"}

    async def _planner(self, query, db=None):
        print("planning what to write")
        prompt = report_plan(query, db)
        res = self.model.completion(prompt)
        return res

    def _task_handler(self, tasks):
        print("handling tasks")
        i = 0
        final_report = ""
        for task in tasks:
            t = task.get("task", "")
            data = task.get("data", "")
            print(t, data)
            source = self.get_source(data)
            print(source)
            prompt = report_task(tasks, t, source)
            res = self.model.completion(prompt)
            res = self._extract_response(res)
            print(res)
            red_flag = False
            try:
                res = json.loads(res)
            except json.JSONDecodeError:
                red_flag = True
                res = {"short_summary": "<ERROR>", "content": "<ERROR>"}
            if not red_flag:
                tasks[i]["content"] = res["short_summary"]
                final_report += res["content"]
                final_report += "\n"
            i += 1
        return final_report

    def _get_relevant_data(self):
        pass
