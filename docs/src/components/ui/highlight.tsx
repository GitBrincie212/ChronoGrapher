import type {JSX} from "react";

type Color = "info" | "warn" | "idea" | "error" | "success" | "neutral";

const variants: Record<Color, string> = {
    "info": "text-fd-info bg-fd-info/10",
    "warn": "text-fd-warning bg-fd-warning/10",
    "error": "text-fd-error bg-fd-error/10",
    "idea": "text-fd-idea bg-fd-idea/10",
    "success": "text-fd-success bg-fd-success/10",
    "neutral": "text-fd-foreground bg-fd-foreground/10",
}

export default function Highlight(props: {
    color: Color
    children: JSX.Element;
    mono?: boolean;
}) {
    return <mark className={`${variants[props.color]} px-1 font-semibold ${props.mono ? "font-mono" : ""}`}>{props.children}</mark>
}