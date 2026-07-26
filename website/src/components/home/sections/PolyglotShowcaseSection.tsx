import SectionWire from "@/components/home/SectionWire";
import Highlight from "@/components/ui/highlight";
import PolyglotIllustration from "@/assets/Polyglot-Section Illustration.png"
import Image from "next/image";
import {DynamicCodeBlock} from "fumadocs-ui/components/dynamic-codeblock";
import styles from "./PolyglotShowcaseSection.module.css"
import React from "react";

const POLYGLOT_TITLE_EXAMPLES = [
    "\"Hello ChronoGrapher!\" – Simple Hello World Script",
    "Complex Workflow Orchestration Made Easy & Intuitive",
    "Deep & Granular Observability Via TaskHooks"
]

const POLYGLOT_CODE_EXAMPLES = [
    "use chronographer::prelude::*;\n\n" +
    "// Replace MyErrors with your own application-specific ones\n" +
    "#[task(schedule = every!(2s))]\n" +
    "async fn MyTask(ctx: &TaskFrameContext) -> Result<(), MyErrors> {\n" +
    "    println!(\"Hello ChronoGrapher!\");\n" +
    "    Ok(())\n" +
    "}\n\n" +
    "#[chronographer::main]\n" +
    "async fn main(scheduler: DefaultScheduler<MyErrors>) {\n" +
    "    let task_inst = MyTask::instance();\n" +
    "    scheduler.schedule(task_inst).await.unwrap();\n" +
    "}",

    "use chronographer::prelude::*;\n\n" +
    "#[task(schedule = every!(2s))]\n" +
    "#[workflow(]\n" +
    "   timeout(20s), // Applies to the entire workflow\n" +
    "   fallback(TaskFrameHandlerAPI, TaskFrameHandlerDB),\n" +
    "   retry(5, 1s, when = [MyErrors::IO, MyErrors::InvalidAPI(408 | 425 | 500)]),\n" +
    "   timeout(5s) // Applies to our code only\n" +
    ")]\n" +
    "async fn HealthCheckupTask(ctx: &TaskFrameContext) -> Result<(), MyErrors> {\n" +
    "    // <...>\n" +
    "}\n\n" +
    "// <...>",

    "use chronographer::prelude::*;\n\n" +
    "event!(OnMyCustomEvent(String, u8))\n\n" +
    "#[derive(Default)]\n" +
    "pub struct MyTaskHook;\n\n" +
    "#[hook]\n" +
    "impl MyTaskHook {\n" +
    "  async fn OnTaskStart(&self, ctx: &TaskHookContext) { /* ... */ }\n" +
    "  async fn OnTaskEnd(&self, ctx: &TaskHookContext, error: Option<&'a dyn TaskError>) { /* ... */}\n" +
    "  async fn OnMyCustomEvent(&self, ctx: &TaskHookContext, param1: String, param2: u8) { /* ... */}\n" +
    "  async fn OnMyGenericEvent<OnRetryAttemptStart>(&self, ctx: &TaskHookContext) { /* ... */}\n\n" +
    "  #[hook(!default)]\n" +
    "  async fn OnMyGenericEvent<E: ChildTaskFrameEvents>(&self, ctx: &TaskHookContext) { /* ... */}\n\n" +
    "  #[hook(!default)]\n" +
    "  async fn __anonymous__<E: MyEventGroup>(&self, ctx: &TaskHookContext) { /* ... */}\n" +
    "}\n\n" +
    "#[task(schedule = every!(2s))]\n" +
    "#[hook(auto(MyTaskHook::default()))]\n" +
    "async fn MyTask(ctx: &TaskFrameContext) -> Result<(), MyErrors> {\n" +
    "    // <...>\n" +
    "    ctx.emit::<OnMyCustomEvent>(\"Test\".to_string(), 1)\n" +
    "    Ok(())\n" +
    "}\n\n" +
    "// <...>",

    "use chronographer::prelude::*;\n\n" +
    "// Replace MyErrors with your own application-specific ones\n" +
    "#[task(schedule = every!(2s))]\n" +
    "async fn MyTask(ctx: &TaskFrameContext) -> Result<(), MyErrors> {\n" +
    "    println!(\"Hello ChronoGrapher!\");\n" +
    "    Ok(())\n" +
    "}\n\n" +
    "#[chronographer::main]\n" +
    "async fn main(scheduler: DefaultScheduler<MyErrors>) {\n" +
    "    let task_inst = MyTask::instance();\n" +
    "    scheduler.schedule(task_inst).await.unwrap();\n" +
    "}",

    "use chronographer::prelude::*;\n\n" +
    "// Replace MyErrors with your own application-specific ones\n" +
    "#[task(schedule = every!(2s))]\n" +
    "async fn MyTask(ctx: &TaskFrameContext) -> Result<(), MyErrors> {\n" +
    "    println!(\"Hello ChronoGrapher!\");\n" +
    "    Ok(())\n" +
    "}\n\n" +
    "#[chronographer::main]\n" +
    "async fn main(scheduler: DefaultScheduler<MyErrors>) {\n" +
    "    let task_inst = MyTask::instance();\n" +
    "    scheduler.schedule(task_inst).await.unwrap();\n" +
    "}",

    "use chronographer::prelude::*;\n\n" +
    "// Replace MyErrors with your own application-specific ones\n" +
    "#[task(schedule = every!(2s))]\n" +
    "async fn MyTask(ctx: &TaskFrameContext) -> Result<(), MyErrors> {\n" +
    "    println!(\"Hello ChronoGrapher!\");\n" +
    "    Ok(())\n" +
    "}\n\n" +
    "#[chronographer::main]\n" +
    "async fn main(scheduler: DefaultScheduler<MyErrors>) {\n" +
    "    let task_inst = MyTask::instance();\n" +
    "    scheduler.schedule(task_inst).await.unwrap();\n" +
    "}",
]

export function PolyglotShowcaseAccordionBtn(props: {
  index: number;
  activeIndex: number;
  setActiveIndex: React.Dispatch<React.SetStateAction<number>>;
}) {
  return (
      <button type={"button"} className={
        "h-full w-64 rounded-sm border border-fd-brand-primary/50 cursor-pointer relative transition-all duration-300 " +
          (props.index === props.activeIndex ? "w-full border-fd-brand-primary" : "")
      } onClick={() => {
        props.setActiveIndex(props.index);
      }}>
        <div className={
          `${props.index === props.activeIndex ? "opacity-100" : "opacity-0"} transition 
          rounded-[0.2rem] duration-300 ${styles.polyglotStrippedBG}`}
        />
        <div className={
          `-z-10 absolute blur-[0.08rem] top-0 scale-105 scale-y-150 ${props.index === props.activeIndex ? "opacity-50" : "opacity-0"} transition 
          rounded-[0.2rem] delay-25 ${styles.polyglotStrippedBG} ${styles.glowPolyglotStrippedBG}`}
        />
      </button>
  );
}

export default function PolyglotShowcaseSection() {
  const [activeIndex, setActiveIndex] = React.useState(0);

  return (
    <SectionWire
      leftRight={false}
      height={28}
      title={
        <>
          Learn Once, <u>Use In Any Programming Language</u>
        </>
      }
      description={
        <>
          <span className={"opacity-60"}>
            Designed to be used in any programming language and ecosystem,
            easily integrating with various established libraries / frameworks /
            platforms.
          </span>
          <span> </span>
          <Highlight color={"primary"}>
            No more polyglot fragmentation
          </Highlight>
        </>
      }
      icon={
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 512 512">
          <path d="M0 0h512v512H0z" fill="none" />
          <path fill="currentColor" d="m478.33 433.6l-90-218a22 22 0 0 0-40.67 0l-90 218a22 22 0 1 0 40.67 16.79L316.66 406h102.67l18.33 44.39A22 22 0 0 0 458 464a22 22 0 0 0 20.32-30.4ZM334.83 362L368 281.65L401.17 362Zm-66.99-19.08a22 22 0 0 0-4.89-30.7c-.2-.15-15-11.13-36.49-34.73c39.65-53.68 62.11-114.75 71.27-143.49H330a22 22 0 0 0 0-44H214V70a22 22 0 0 0-44 0v20H54a22 22 0 0 0 0 44h197.25c-9.52 26.95-27.05 69.5-53.79 108.36c-31.41-41.68-43.08-68.65-43.17-68.87a22 22 0 0 0-40.58 17c.58 1.38 14.55 34.23 52.86 83.93c.92 1.19 1.83 2.35 2.74 3.51c-39.24 44.35-77.74 71.86-93.85 80.74a22 22 0 1 0 21.07 38.63c2.16-1.18 48.6-26.89 101.63-85.59c22.52 24.08 38 35.44 38.93 36.1a22 22 0 0 0 30.75-4.9Z" />
        </svg>
      }
      iconColorClass={"bg-fd-brand-primary/5 dark:bg-fd-brand-primary/20 border-fd-brand-primary text-fd-brand-primary"}
    >
      <div className={"w-full h-full flex justify-between pl-24"}>
        <div className={"w-3xl flex flex-col -mt-24"}>
          <h1 className={"text-lg text-start font-bold mb-1"}>{POLYGLOT_TITLE_EXAMPLES[activeIndex]}</h1>
          <DynamicCodeBlock lang={"rust"} codeblock={{
            "data-line-numbers": true,
            keepBackground: false,
            className: "w-full h-full mb-3 dark:bg-fd-background-100p border-fd-brand-primary/40 rounded-lg text-start overflow-scroll"
          }} code={POLYGLOT_CODE_EXAMPLES[activeIndex]} />
          <div className={"flex justify-center w-full h-4 gap-2 pr-8 pl-1"}>
            <PolyglotShowcaseAccordionBtn index={0} activeIndex={activeIndex} setActiveIndex={setActiveIndex} />
            <PolyglotShowcaseAccordionBtn index={1} activeIndex={activeIndex} setActiveIndex={setActiveIndex} />
            <PolyglotShowcaseAccordionBtn index={2} activeIndex={activeIndex} setActiveIndex={setActiveIndex} />
            <PolyglotShowcaseAccordionBtn index={3} activeIndex={activeIndex} setActiveIndex={setActiveIndex} />
            <PolyglotShowcaseAccordionBtn index={4} activeIndex={activeIndex} setActiveIndex={setActiveIndex} />
            <PolyglotShowcaseAccordionBtn index={5} activeIndex={activeIndex} setActiveIndex={setActiveIndex} />
          </div>
        </div>
        <div className={"w-fit h-full -mt-8"}>
          <Image className={"w-fit h-[calc(100%+2.5rem)] object-cover"} src={PolyglotIllustration} alt={""} unoptimized={true} />
        </div>
      </div>
    </SectionWire>
  );
}
