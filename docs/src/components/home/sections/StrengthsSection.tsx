import React, {useEffect} from "react";

function StrengthShowcase1() {
    const codes = [
`use chronographer::prelude::*;

#[task(interval = 1.5s)]
async fn mytask(ctx: &TaskContext) -> Result<(), TaskError> {
    println!("Hello ChronoGrapher!");
    Ok(())
}

#[chronographer::main]
async fn main() {
    let task_inst = mytask();
    CHRONOGRAPHER_SCHEDULER.schedule(task_inst).await;
}`,

    ]

    return (
        <div className={
            "w-full ring-2 ring-offset-8 ring-offset-fd-background ring-fd-foreground " +
            "h-full rounded-lg bg-fd-background-light overflow-hidden"
        }>
            <div className={"w-full h-fit flex"}>
                {/* CODE HERE */}
            </div>
        </div>
    );
}

const strengthsList = [
    {
        startColor: "var(--color-fd-info)",
        endColor: "oklch(0.512 0.152 259.439)",
        title: "Multilingual-Level Scheduling",
        description: "One unified platform connected across your favourite programming languages, No more glue-code, no more leaky patterns and no more brittleness. Everything as it's meant to be.",
        icon: <path fill="none" stroke="url(#strengths-gradient)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="m5 8l6 6m-7 0l6-6l2-3M2 5h12M7 2h1m14 20l-5-10l-5 10m2-4h6"/>,
        content: <StrengthShowcase1 />
    },
    {
        startColor: "var(--color-fd-brand-primary)",
        endColor: "var(--color-fd-brand-secondary)",
        title: "Hyper-Extensible Architecture",
        description: "Swap every component without incurring massive rewrites, clocks, dispatchers, stores and even the engine. ChronoGrapher adapts to your infrastructure, not the other way around.",
        icon: <path fill="none" stroke="url(#strengths-gradient)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M15.39 4.39a1 1 0 0 0 1.68-.474a2.5 2.5 0 1 1 3.014 3.015a1 1 0 0 0-.474 1.68l1.683 1.682a2.414 2.414 0 0 1 0 3.414L19.61 15.39a1 1 0 0 1-1.68-.474a2.5 2.5 0 1 0-3.014 3.015a1 1 0 0 1 .474 1.68l-1.683 1.682a2.414 2.414 0 0 1-3.414 0L8.61 19.61a1 1 0 0 0-1.68.474a2.5 2.5 0 1 1-3.014-3.015a1 1 0 0 0 .474-1.68l-1.683-1.682a2.414 2.414 0 0 1 0-3.414L4.39 8.61a1 1 0 0 1 1.68.474a2.5 2.5 0 1 0 3.014-3.015a1 1 0 0 1-.474-1.68l1.683-1.682a2.414 2.414 0 0 1 3.414 0z"/>,
        content: <StrengthShowcase1 />
    },
    {
        startColor: "var(--color-fd-neutral)",
        endColor: "var(--color-fd-card-foreground)",
        title: "Highly Adaptive Durability",
        description: "Configure your level of durability to persist state automatically, resume workflows where they left off. Crashes, restarts, and infrastructure failures don't mean lost work.",
        icon: <path fill="none" stroke="url(#strengths-gradient)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M22 12H2m3.45-6.89L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11M6 16h.01M10 16h.01"/>,
        content: <StrengthShowcase1 />
    },
    {
        startColor: "oklch(0.783 0.146 85.151)",
        endColor: "var(--color-fd-idea)",
        title: "Supercharged Performance",
        description: "Zero-cost abstraction and lightning-fast tickless scheduling algorithms allow millions of tasks in throughput with minimal overhead. Configurable for all your blazing desires.",
        icon: <path fill="none" stroke="url(#strengths-gradient)" strokeWidth="2" d="m5.67 9.914l3.062-4.143c1.979-2.678 2.969-4.017 3.892-3.734s.923 1.925.923 5.21v.31c0 1.185 0 1.777.379 2.148l.02.02c.387.363 1.003.363 2.236.363c2.22 0 3.329 0 3.704.673l.018.034c.354.683-.289 1.553-1.574 3.29l-3.062 4.144c-1.98 2.678-2.969 4.017-3.892 3.734s-.923-1.925-.923-5.21v-.31c0-1.185 0-1.777-.379-2.148l-.02-.02c-.387-.363-1.003-.363-2.236-.363c-2.22 0-3.329 0-3.703-.673l-.019-.034c-.354-.683.289-1.552 1.574-3.29Z"/>,
        content: <StrengthShowcase1 />
    },
    {
        startColor: "var(--color-fd-success)",
        endColor: "oklch(0.605 0.173 148.241)",
        title: "Scales With Your Ambitions",
        description: "From home laptops to full distributed clusters managed with Kubernetes, monitored with Prometheus and Scheduled with ChronoGrapher, make enterprise scale dreams possible.",
        icon: <g fill="none" stroke="url(#strengths-gradient)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2"><path d="M16 3h5v5m-4 13h2a2 2 0 0 0 2-2m0-7v3m0-12l-5 5M3 7V5a2 2 0 0 1 2-2m0 18l4.144-4.144a1.21 1.21 0 0 1 1.712 0L13 19M9 3h3"/><rect width="10" height="10" x="3" y="11" rx="1"/></g>,
        content: <StrengthShowcase1 />
    }
]

function StrengthToggle(props: { setStrengthIdx: React.Dispatch<React.SetStateAction<number>>, strengthIdx: number, idx: number }) {
    return (
        <button
            type="button"
            className={
                `cursor-pointer h-full rounded-full transition-all duration-300 ring-offset-2 ring-1 py-2
                ring-offset-fd-background ${props.strengthIdx === props.idx 
                    ? `w-48 bg-(--stop-color-start) ring-(--stop-color-start)` 
                    : "w-12 bg-fd-foreground/25 ring-transparent"
                }
            `}
            onClick={() => props.setStrengthIdx(props.idx)}
        ></button>
    );
}

export default function StrengthsSection() {
    const strengthsContainer = React.useRef<HTMLDivElement>(null);
    const [strengthIdx, setStrengthIdx] = React.useState(0);

    useEffect(() => {
        if (!strengthsContainer.current) return;
        strengthsContainer.current.style.setProperty("--stop-color-start", strengthsList[strengthIdx].startColor);
        strengthsContainer.current.style.setProperty("--stop-color-end", strengthsList[strengthIdx].endColor);
    }, [strengthIdx]);

    return (
        <div className={"w-screen flex justify-between h-96 mt-18 px-32 gap-32"}>
            <div className={"w-fit flex flex-col justify-center h-full gap-2"} ref={strengthsContainer}>
                <div className={"flex items-center gap-2"}>
                    <div className={
                        "size-14 flex justify-center items-center bg-linear-to-b from-(--stop-color-start) " +
                        "to-(--stop-color-end) transition duration-200 rounded-lg p-0.75"
                    }>
                        <div className={"w-full h-full bg-fd-muted rounded-md"}>
                            <div className={"w-full h-full flex justify-center items-center text-5xl p-2"}>
                                {/** biome-ignore lint/a11y/noSvgWithoutTitle: No need, as they are icons */}
                                <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                                    <defs>
                                        <linearGradient id="strengths-gradient" x1="0" x2="0" y1="0" y2="1">
                                            <stop style={{stopColor: `var(--stop-color-start)`}} offset="0%" />
                                            <stop style={{stopColor: `var(--stop-color-end)`}} offset="100%" />
                                        </linearGradient>
                                    </defs>
                                    {strengthsList[strengthIdx].icon}
                                </svg>
                            </div>
                        </div>
                    </div>
                    <div className={"text-2xl font-semibold"}>{strengthsList[strengthIdx].title}</div>
                </div>
                <div className={"w-[70ch] opacity-50"}>
                    {strengthsList[strengthIdx].description}
                </div>
                <div className={"flex w-full flex-1 max-h-2 gap-1.5 mt-2"}>
                    {strengthsList.map((_, idx) => (
                        <StrengthToggle setStrengthIdx={setStrengthIdx} strengthIdx={strengthIdx} idx={idx} key={`strengths-toggle`} />
                    ))}
                </div>
            </div>
            {strengthsList[strengthIdx].content}
        </div>
    );
}