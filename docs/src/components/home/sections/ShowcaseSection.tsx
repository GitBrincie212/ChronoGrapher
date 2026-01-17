/** biome-ignore-all lint/a11y/noSvgWithoutTitle: Shut up */

import React, { type JSX } from "react";
import ShikiHighlighter from "react-shiki";
import {useFumadocsTheme} from "@/components/theme";

const PROGRAMMING_LANG_SELECTORS = [
  {
    icon: (
      <path
        fill="url(#showcase-gradient-0)"
        d="m23.835 11.703l-1.008-.623l-.028-.294l.866-.807a.348.348 0 0 0-.116-.578l-1.106-.414a9 9 0 0 0-.087-.285l.69-.96a.346.346 0 0 0-.226-.544l-1.166-.19a9 9 0 0 0-.14-.261l.49-1.076a.34.34 0 0 0-.028-.336a.35.35 0 0 0-.3-.154l-1.185.041a7 7 0 0 0-.188-.227l.273-1.153a.347.347 0 0 0-.417-.417l-1.153.273l-.228-.188l.041-1.184a.344.344 0 0 0-.49-.328l-1.076.49l-.262-.14l-.19-1.167a.348.348 0 0 0-.545-.226l-.96.69a9 9 0 0 0-.285-.086L14.597.453a.348.348 0 0 0-.578-.116l-.807.867a9 9 0 0 0-.294-.028L12.295.168a.346.346 0 0 0-.59 0l-.623 1.008l-.294.028L9.98.337a.346.346 0 0 0-.578.116l-.414 1.106l-.285.086l-.959-.69a.348.348 0 0 0-.545.226l-.19 1.167a9 9 0 0 0-.262.14l-1.076-.49a.346.346 0 0 0-.49.328l.041 1.184a8 8 0 0 0-.228.187l-1.153-.272a.347.347 0 0 0-.417.417l.271 1.153l-.186.227l-1.184-.042a.346.346 0 0 0-.328.49l.49 1.077a9 9 0 0 0-.14.262l-1.166.19a.348.348 0 0 0-.226.544l.69.958l-.087.286l-1.106.414a.348.348 0 0 0-.116.578l.866.807a9 9 0 0 0-.028.294l-1.008.623a.344.344 0 0 0 0 .59l1.008.623q.012.147.028.294l-.866.807a.346.346 0 0 0 .116.578l1.106.415q.042.144.087.285l-.69.959a.345.345 0 0 0 .227.544l1.166.19q.069.132.14.262l-.49 1.076a.346.346 0 0 0 .328.49l1.183-.041q.093.115.187.227l-.27 1.154a.346.346 0 0 0 .416.417l1.153-.272q.113.096.228.187l-.041 1.184a.344.344 0 0 0 .49.327l1.076-.49q.13.073.262.14l.19 1.167a.348.348 0 0 0 .545.227l.959-.69a9 9 0 0 0 .285.086l.414 1.107a.345.345 0 0 0 .578.115l.808-.865l.294.03l.623 1.006a.347.347 0 0 0 .59 0l.623-1.007q.148-.013.294-.03l.807.866a.348.348 0 0 0 .578-.115l.414-1.107a9 9 0 0 0 .285-.087l.959.69a.345.345 0 0 0 .545-.226l.19-1.166l.262-.14l1.076.49a.347.347 0 0 0 .49-.328l-.041-1.184a7 7 0 0 0 .227-.187l1.153.272a.347.347 0 0 0 .417-.416l-.272-1.155q.095-.112.187-.227l1.184.041a.344.344 0 0 0 .328-.49l-.49-1.076q.072-.13.141-.262l1.166-.19a.348.348 0 0 0 .226-.544l-.69-.959l.087-.285l1.106-.414a.346.346 0 0 0 .116-.579l-.866-.807q.016-.147.028-.294l1.008-.624a.344.344 0 0 0 0-.589zm-6.742 8.355a.714.714 0 0 1 .299-1.396a.714.714 0 1 1-.3 1.396zm-.342-2.314a.65.65 0 0 0-.771.5l-.358 1.669a8.7 8.7 0 0 1-3.619.78a8.7 8.7 0 0 1-3.695-.815L7.95 18.21a.65.65 0 0 0-.772-.5l-1.473.317a9 9 0 0 1-.761-.898h7.167c.081 0 .136-.014.136-.088v-2.536c0-.074-.054-.088-.136-.088h-2.096v-1.608h2.268c.206 0 1.106.059 1.393 1.209c.09.353.288 1.504.424 1.873c.134.413.683 1.238 1.268 1.238h3.572a1 1 0 0 0 .13-.013a9 9 0 0 1-.813.952zm-9.914 2.28a.714.714 0 1 1-.3-1.396a.714.714 0 0 1 .3 1.396M4.117 8.997a.714.714 0 1 1-1.303.58a.714.714 0 0 1 1.304-.58m-.834 1.981l1.534-.682a.65.65 0 0 0 .33-.858l-.316-.715h1.244v5.602H3.567a8.8 8.8 0 0 1-.284-3.348zm6.734-.543V8.784h2.96c.153 0 1.08.177 1.08.87c0 .574-.712.78-1.296.78zm10.757 1.486q0 .329-.024.651h-.9c-.09 0-.127.059-.127.148v.413c0 .973-.548 1.184-1.03 1.238c-.457.052-.964-.191-1.027-.472c-.27-1.518-.72-1.843-1.43-2.403c.882-.56 1.799-1.386 1.799-2.492c0-1.193-.82-1.945-1.377-2.315c-.783-.516-1.65-.62-1.883-.62H5.468a8.77 8.77 0 0 1 4.907-2.77l1.098 1.152a.65.65 0 0 0 .918.02l1.227-1.173a8.78 8.78 0 0 1 6.004 4.276l-.84 1.898a.65.65 0 0 0 .33.859l1.618.718q.042.43.042.872zm-9.3-9.6a.713.713 0 1 1 .984 1.032a.714.714 0 0 1-.984-1.031m8.339 6.71a.71.71 0 0 1 .939-.362a.714.714 0 1 1-.94.364z"
      />
    ),
  },
  {
    icon: (
      <>
        <g fill="url(#showcase-gradient-1)" clipPath="url(#SVGXv8lpc2Y)">
          <path d="M11.914 0C5.82 0 6.2 2.656 6.2 2.656l.007 2.752h5.814v.826H3.9S0 5.789 0 11.969s3.403 5.96 3.403 5.96h2.03v-2.867s-.109-3.42 3.35-3.42h5.766s3.24.052 3.24-3.148V3.202S18.28 0 11.913 0M8.708 1.85c.578 0 1.046.47 1.046 1.052c0 .581-.468 1.051-1.046 1.051s-1.046-.47-1.046-1.051c0-.582.467-1.052 1.046-1.052" />
          <path d="M12.087 24c6.092 0 5.712-2.656 5.712-2.656l-.007-2.752h-5.814v-.826h8.123s3.9.445 3.9-5.735s-3.404-5.96-3.404-5.96h-2.03v2.867s.109 3.42-3.35 3.42H9.452s-3.24-.052-3.24 3.148v5.292S5.72 24 12.087 24m3.206-1.85c-.579 0-1.046-.47-1.046-1.052c0-.581.467-1.051 1.046-1.051c.578 0 1.046.47 1.046 1.051c0 .582-.468 1.052-1.046 1.052" />
        </g>
        <defs>
          <clipPath id="SVGXv8lpc2Y">
            <path fill="#fff" d="M0 0h24v24H0z" />
          </clipPath>
        </defs>
      </>
    ),
  },
  {
    icon: (
      <path
        fill="url(#showcase-gradient-2)"
        d="M10.82 8.427c-.76-1.085-1.046-1.872-1.108-2.445c-.059-.54.073-.97.335-1.383c.283-.447.708-.862 1.248-1.353c.505-.458 1.137-1.003 1.633-1.62L11.37.374c-.416.517-.928.947-1.418 1.391c-.534.485-1.149 1.062-1.593 1.764c-.466.735-.75 1.611-.635 2.67c.112 1.026.588 2.132 1.458 3.375zm1.25-3.03c.284-.684.861-1.37 1.78-2.156l1.3 1.518c-.831.713-1.129 1.153-1.235 1.407c-.104.25-.035.499.035.747c.084.296.223.79.214 1.322c-.012.706-.254 1.452-.832 2.32l-1.664-1.11c.422-.632.492-1.01.496-1.243c.005-.295-.072-.46-.197-.96c-.12-.478-.21-1.097.102-1.845M6.141 10c.47-.314 1.05-.474 1.592-.604L7.266 7.45c-.813.196-1.551.46-2.112.807C4.644 8.573 4 9.136 4 10c0 .726.486 1.306 1.065 1.686a2.5 2.5 0 0 0-.065.572c0 .98.418 1.807 1.143 2.42c-.107.3-.155.634-.14.968c-.777.151-1.536.373-2.17.664c-.432.198-.858.451-1.19.777c-.334.329-.643.804-.643 1.413c0 .427.189.772.374 1.01c.189.242.432.449.684.624c.506.351 1.184.669 1.959.936c1.558.538 3.669.93 5.983.93c3.327 0 5.663-.384 7.19-.782a13 13 0 0 0 1.704-.561c.21-.088.42-.181.616-.297c.01-.006-1.028-1.715-1.024-1.718c-.564.294-1.188.48-1.8.64c-1.349.352-3.513.718-6.686.718c-2.104 0-3.993-.358-5.33-.82c-.532-.184-1.088-.398-1.541-.74c.115-.09.29-.199.538-.312c.558-.256 1.323-.466 2.14-.591q.344.365.814.64c.965.562 2.292.823 3.879.823c1.31 0 2.258-.132 2.898-.274c.356-.078.714-.167 1.042-.328c.003-.002-.881-1.795-.881-1.795q-.291.105-.593.17c-.489.109-1.29.227-2.466.227c-1.413 0-2.336-.24-2.871-.551c-.426-.248-.589-.524-.622-.824c.868.253 1.895.375 3.056.375c1.463 0 2.518-.143 3.228-.297c.386-.083.775-.179 1.134-.346c.02-.01-.857-1.807-.857-1.807a4.4 4.4 0 0 1-.7.198c-.56.121-1.473.252-2.806.252c-1.603 0-2.67-.264-3.302-.623c-.471-.268-.678-.568-.74-.89c1.24.326 2.805.513 4.48.513c1.817 0 3.502-.22 4.783-.598l-.566-1.918c-1.054.311-2.54.516-4.217.516c-1.724 0-3.245-.216-4.302-.542c-.5-.153-.846-.316-1.056-.458m11.448.746c1.213-.11 1.66.188 1.804.342c.149.16.201.408.136.67c-.116.464-.443.914-.973 1.328c-.529.413-1.2.74-1.873.965l.632 1.897c.828-.276 1.718-.698 2.471-1.285c.75-.586 1.424-1.385 1.683-2.42c.185-.74.112-1.741-.614-2.52c-.73-.782-1.909-1.109-3.447-.969z"
      />
    ),
  },
  {
    icon: (
      <path
        fill="url(#showcase-gradient-3)"
        d="M12 23.956c-.342 0-.66-.089-.957-.243l-3.029-1.738c-.455-.242-.227-.33-.09-.374c.614-.198.728-.242 1.366-.595c.068-.044.16-.022.228.022l2.323 1.343c.09.044.205.044.273 0l9.087-5.084c.09-.044.136-.132.136-.242V6.899c0-.11-.045-.198-.136-.242l-9.087-5.061c-.091-.044-.205-.044-.273 0L2.754 6.657c-.091.044-.137.154-.137.242v10.146c0 .088.046.198.137.242l2.482 1.387c1.344.66 2.186-.11 2.186-.88V7.78c0-.132.114-.264.274-.264h1.161c.137 0 .273.11.273.264v10.013c0 1.739-.979 2.751-2.687 2.751c-.524 0-.934 0-2.095-.55l-2.391-1.32A1.85 1.85 0 0 1 1 17.067V6.921c0-.66.364-1.276.957-1.606L11.044.23a2.1 2.1 0 0 1 1.912 0l9.088 5.084c.592.33.956.946.956 1.606v10.146c0 .66-.364 1.276-.956 1.607l-9.087 5.083a2.4 2.4 0 0 1-.957.198m2.801-6.977c-3.985 0-4.805-1.76-4.805-3.257c0-.132.114-.264.273-.264h1.184c.137 0 .25.088.25.22c.183 1.166.707 1.738 3.121 1.738c1.913 0 2.733-.418 2.733-1.408c0-.572-.228-.99-3.211-1.276c-2.483-.243-4.031-.77-4.031-2.685c0-1.783 1.548-2.84 4.145-2.84c2.915 0 4.35.969 4.532 3.082a.35.35 0 0 1-.069.198c-.045.044-.113.088-.182.088h-1.184a.265.265 0 0 1-.25-.198c-.274-1.21-.98-1.607-2.847-1.607c-2.096 0-2.346.704-2.346 1.233c0 .638.296.836 3.12 1.188c2.801.352 4.122.858 4.122 2.75c-.023 1.938-1.662 3.038-4.555 3.038"
      />
    ),
  },
];

const EXAMPLE_SELECTORS = [
  {
    activeColor: "bg-fd-info ring-fd-info",
    code: `use chronographer::prelude::*;

#[task(interval = 4s)]
async fn mytask(ctx: &TaskContext) -> Result<(), TaskError> {
  println!("Hello World");
  Ok(())
}

#[chronographer::main]
async fn main() {
  let task = mytask();
  let _ = CHRONOGRAPHER_SCHEDULER.schedule(&task).await;
}`,
  },
  {
    activeColor: "bg-fd-warning ring-fd-warning",
    code: `use chronographer::prelude::*;

#[derive(Scheduler)]
#[scheduler_config(
  task_identifier = usize,
  dispatcher = DefaultTaskDispatcher,
  clock = VirtualClock,
  engine = MySchedulerEngine,
  store = MySchedulerTaskStore
)]
struct MySimulationScheduler;`,
  },
  {
    activeColor: "bg-fd-error ring-fd-error",
    code: `use chronographer::prelude::*;

#[task(interval = 4s)]
async fn mytask(ctx: &TaskContext) -> Result<(), TaskError> {
  println!("Hello World");
  Ok(())
}

#[chronographer::main]
async fn main() {
  let task = mytask();
  let _ = CHRONOGRAPHER_SCHEDULER.schedule(&task).await;
}`,
  },
  {
    activeColor: "bg-fd-success ring-fd-success",
    code: `use chronographer::prelude::*;

#[task(interval = 4s)]
async fn mytask(ctx: &TaskContext) -> Result<(), TaskError> {
  println!("Hello World");
  Ok(())
}

#[chronographer::main]
async fn main() {
  let task = mytask();
  let _ = CHRONOGRAPHER_SCHEDULER.schedule(&task).await;
}`,
  }
]

function ShowcaseLangSelection(props: {
  icon: JSX.Element;
  activeIndex: number;
  setActiveIndex: React.Dispatch<React.SetStateAction<number>>;
  idx: number;
}) {
  return (
    <button type={"button"}
      className={
        `transition-all duration-300 size-12 rounded-md p-0.5 cursor-pointer 
        ${props.activeIndex !== props.idx ? "opacity-35 scale-90" : "scale-110"}
      `}
      onClick={() => props.setActiveIndex(props.idx)}
      style={{
        background:
          props.activeIndex === props.idx
            ? `linear-gradient(to bottom, var(--color-fd-brand-primary), color-mix(in lch, var(--color-fd-background-dark) 50%, var(--color-fd-brand-primary)))`
            : "linear-gradient(to bottom, var(--color-fd-foreground), color-mix(in lch, var(--color-fd-foreground) 30%, var(--color-fd-background)))",
      }}
    >
      <div className={"size-full bg-fd-background rounded-[0.3rem]"}>
        <div
          className={
            "size-full bg-clip-text flex items-center justify-center text-transparent text-4xl"
          }
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="1em"
            height="1em"
            viewBox="0 0 24 24"
          >
            <defs>
              <linearGradient
                id={`showcase-gradient-${props.idx}`}
                x1="0"
                x2="0"
                y1="0"
                y2="1"
              >
                <stop
                  offset="0%"
                  stopColor={
                    props.activeIndex === props.idx
                      ? "var(--color-fd-brand-primary)"
                      : "var(--color-fd-foreground)"
                  }
                />
                <stop
                  offset="100%"
                  stopColor={
                    props.activeIndex === props.idx
                      ? "color-mix(in lch, var(--color-fd-background-dark) 65%, var(--color-fd-brand-primary))"
                      : "color-mix(in lch, var(--color-fd-foreground) 30%, var(--color-fd-background))"
                  }
                />
              </linearGradient>
            </defs>
            {props.icon}
          </svg>
        </div>
      </div>
    </button>
  );
}

function ShowcaseExampleSelector(props: {
  idx: number;
  exampleSelectorIndex: number;
  setExampleSelectorIndex: React.Dispatch<React.SetStateAction<number>>;
  activeColor: string;
}) {
  return (
      <button type={"button"} className={
        `cursor-pointer transition-all duration-300 ring-2 ring-offset-4 ring-offset-fd-background ${props.idx === props.exampleSelectorIndex
            ? `${props.activeColor} h-full`
            : "ring-fd-foreground/50 bg-fd-muted h-1/3"
        } w-full rounded-full`
      } onClick={() => {props.setExampleSelectorIndex(props.idx)}} />
  );
}

export default function ShowcaseSection() {
  const [activeLangIndex, setActiveLangIndex] = React.useState<number>(0);
  const [exampleSelectorIndex, setExampleSelectorIndex] = React.useState<number>(0);

  return (
    <div className={"w-screen min-h-3xl mt-18 mb-12"}>
      <h1 className={"text-4xl font-bold text-center w-full"}>Getting Started Showcase</h1>
      <div className={"text-xl opacity-50 w-full text-center mb-8"}>Basic introductory script to ChronoGrapher's platform</div>
      <div className={"w-full h-full flex justify-center items-stretch gap-8"}>
        <div className={"flex flex-col w-fit gap-6 py-6"}>
          {PROGRAMMING_LANG_SELECTORS.map((lang, idx) => (
              <ShowcaseLangSelection
                  activeIndex={activeLangIndex}
                  setActiveIndex={setActiveLangIndex}
                  idx={idx}
                  {...lang}
                  key={
                    // biome-ignore lint/suspicious/noArrayIndexKey: It is a constant array, so it can't change
                    `lang-selection-${idx}`
                  }
              />
          ))}
        </div>
        <div className={"w-3xl bg-fd-muted h-96 rounded-lg [&_.shiki]:bg-transparent! [&_.shiki]:pl-0! ring-1 ring-offset-4 ring-offset-fd-background ring-fd-foreground/25"}>
          <ShikiHighlighter
              language={"rust"}
              theme={useFumadocsTheme() === "dark" ? "github-dark" : "github-light"}
              defaultColor={"dark"}
              showLineNumbers={true}
          >
            {EXAMPLE_SELECTORS[exampleSelectorIndex].code}
          </ShikiHighlighter>
        </div>
        <div className={"w-4 mb-2 min-h-full gap-6 flex flex-col"}>
          {EXAMPLE_SELECTORS.map((selector, idx) => (
              <ShowcaseExampleSelector idx={idx}
                  exampleSelectorIndex={exampleSelectorIndex}
                  setExampleSelectorIndex={setExampleSelectorIndex}
                  {...selector}
                  key={
                    // biome-ignore lint/suspicious/noArrayIndexKey: It is a constant array, so it can't change
                    `xample-selection-${idx}`
                  }
              />
          ))}
        </div>
      </div>
    </div>
  );
}
