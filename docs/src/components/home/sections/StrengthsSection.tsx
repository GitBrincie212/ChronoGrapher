/** biome-ignore-all lint/a11y/noSvgWithoutTitle: Shut up */
import type {JSX} from "react";

function StrengthsShowcase1() {
  return (
      <div className={"w-full h-full bg-fd-foreground rounded-lg"}>

      </div>
  );
}

const StrengthsList = [
  {
    icon: <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2"><path d="M12.409 13.017A5 5 0 0 1 22 15c0 3.866-4 7-9 7c-4.077 0-8.153-.82-10.371-2.462c-.426-.316-.631-.832-.62-1.362C2.118 12.723 2.627 2 10 2a3 3 0 0 1 3 3a2 2 0 0 1-2 2c-1.105 0-1.64-.444-2-1"/><path d="M15 14a5 5 0 0 0-7.584 2"/><path d="M9.964 6.825C8.019 7.977 9.5 13 8 15"/></g></svg>,
    title: "Refined For Developers",
    description: "Crafted for the best possible developer experience in job scheduling and workflow orchestration. Minimal over bloat, simple over complex and emergent over predefined",
    color1: "var(--color-fd-info)",
    color2: "oklch(0.46 0.128 258.548)",
    content: <StrengthsShowcase1 />
  },
  {
    icon: <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M15.39 4.39a1 1 0 0 0 1.68-.474a2.5 2.5 0 1 1 3.014 3.015a1 1 0 0 0-.474 1.68l1.683 1.682a2.414 2.414 0 0 1 0 3.414L19.61 15.39a1 1 0 0 1-1.68-.474a2.5 2.5 0 1 0-3.014 3.015a1 1 0 0 1 .474 1.68l-1.683 1.682a2.414 2.414 0 0 1-3.414 0L8.61 19.61a1 1 0 0 0-1.68.474a2.5 2.5 0 1 1-3.014-3.015a1 1 0 0 0 .474-1.68l-1.683-1.682a2.414 2.414 0 0 1 0-3.414L4.39 8.61a1 1 0 0 1 1.68.474a2.5 2.5 0 1 0 3.014-3.015a1 1 0 0 1-.474-1.68l1.683-1.682a2.414 2.414 0 0 1 3.414 0z"/></svg>,
    title: "Hyper-Level Extensibility",
    description: "Replace any component with YOUR own, ranging from TaskHooks to the scheduler's inner workings. Adapt ChronoGrapher to your growing needs, not the other way around.",
    color1: "var(--color-fd-brand-primary)",
    color2: "var(--color-fd-brand-secondary)",
    content: <StrengthsShowcase1 />
  },
  {
    icon: <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2"><path d="M16 3h5v5m-4 13h2a2 2 0 0 0 2-2m0-7v3m0-12l-5 5M3 7V5a2 2 0 0 1 2-2m0 18l4.144-4.144a1.21 1.21 0 0 1 1.712 0L13 19M9 3h3"/><rect width="10" height="10" x="3" y="11" rx="1"/></g></svg>,
    title: "Scales With Your Ambitions",
    description: "From self-hosted to distributed clusters managed with Kubernetes, monitored with Prometheus and Scheduled with ChronoGrapher. Make enterprise-level dreams possible",
    color1: "var(--color-fd-success)",
    color2: "oklch(0.585 0.164 148.77)",
    content: <StrengthsShowcase1 />
  },
  {
    icon: <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2"><path d="M7 3v5h8"/><path d="M5 21a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2Z"/><path d="M17 21v-8H7v8"/></g></svg>,
    title: "Adaptive Durability",
    description: "Configure the level of reliability, durability in ChronoGrapher is not just one value. Its an entire spectrum affected by the persistence backend engine and other choices YOU control",
    color1: "var(--color-fd-neutral)",
    color2: "var(--color-fd-neutral-dark)",
    content: <StrengthsShowcase1 />
  }
]

function StrengthsIcon(props: {
  icon: JSX.Element;
  color1: string;
  color2: string;
}) {
  return (
      <div className={
        "size-10 border-3 border-fd-background bg-fd-foreground rounded-full ring-4 ring-offset-2 " +
          "ring-offset-fd-foreground ring-fd-background justify-center items-center flex text-2xl text-fd-background"
      }  style={{
        // @ts-expect-error You can do that, lil bro
        "--strengths-color-1": props.color1,
        "--strengths-color-2": props.color2,
      }}>
        {props.icon}
      </div>
  );
}

function StrengthsShowcase(props: {
  title: string;
  description: string;
  content: JSX.Element;
  is_left?: boolean;
  color1: string;
  color2: string;
}) {
  return (
      <div className={`w-full h-96 flex flex-col ${props.is_left ? "items-start" : "items-end"}`} style={{
        // @ts-expect-error You can do that, lil bro
        "--strengths-color-1": props.color1,
        "--strengths-color-2": props.color2,
      }}>
        <div className={"relative contents"}>
          <div className={
            `absolute text-transparent bg-clip-text ${props.is_left ? "text-start" : "text-end"} bg-linear-to-r
           from-(--strengths-color-1) to-(--strengths-color-2) font-semibold text-4xl saturate-150 z-10`
          }>{props.title}</div>
          <div className={
            `text-transparent bg-clip-text ${props.is_left ? "text-start" : "text-end"} bg-linear-to-r
           from-(--strengths-color-1) to-(--strengths-color-2) font-semibold text-4xl blur-sm opacity-50 saturate-150`
          }>{props.title}</div>
        </div>
        <div className={`font-[VioletSans] opacity-50 w-[70ch] ${props.is_left ? "text-start" : "text-end"} mb-4`}>{props.description}</div>
        <div className={"w-full h-96"}>{props.content}</div>
      </div>
  );
}

export default function StrengthsSection() {
  return (
    <div className={"relative w-screen max-h-600 h-max mt-18 flex justify-between px-16 gap-x-24"}>
      <div className={"w-full flex flex-col pt-16 h-full gap-48"}>
        {StrengthsList.map((x, idx) => {
          const key = `strengths-${idx}`;
          if (idx % 2 !== 0) return <StrengthsShowcase is_left key={key} title={""} description={""} color1={""} color2={""} content={<span></span>} />;
          return <StrengthsShowcase is_left key={key} {...x} />
        })}
      </div>
      <div className={"relative h-full w-0.5 bg-fd-foreground flex flex-col items-center py-64 gap-140"}>
        {StrengthsList.map((x, idx) => (
            <StrengthsIcon key={`strengths-${
              // biome-ignore lint/suspicious/noArrayIndexKey: Constant array, no shuffling, no moves
              idx
            }`} {...x} />
        ))}
      </div>
      <div className={"w-full flex flex-col items-end pt-16 h-full gap-48"}>
        {StrengthsList.map((x, idx) => {
          const key = `strengths-${idx}`;
          if (idx % 2 === 0) return <StrengthsShowcase is_left key={key} title={""} description={""} color1={""} color2={""} content={<span></span>} />;
          return <StrengthsShowcase key={key} {...x} />
        })}
      </div>
    </div>
  );
}
