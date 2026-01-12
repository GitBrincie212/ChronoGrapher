"use client";

import hljs from "highlight.js";
import HeroSection from "@/components/home/sections/HeroSection";
import StrengthsSection from "@/components/home/sections/StrengthsSection";
import "highlight.js/styles/github.css";
import rust from "highlight.js/lib/languages/rust";
import { useEffect } from "react";

export default function HomePage() {
  useEffect(() => {
    hljs.registerLanguage("rust", rust);
    document.querySelectorAll(".highlightjs-highlight").forEach((el) => {
      hljs.highlightElement(el as HTMLElement);
    });
  }, []);

  return (
    <main className="overflow-x-hidden">
      <HeroSection />
      <div className={"w-screen min-h-[80rem]"}></div>
    </main>
  );
}
