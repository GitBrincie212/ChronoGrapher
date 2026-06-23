"use client";

import HeroSection from "@/components/home/sections/HeroSection";
import ScalabilityShowcaseSection from "@/components/home/sections/ScalabilityShowcaseSection";
import PolyglotShowcaseSection from "@/components/home/sections/PolyglotShowcaseSection";
import PerformanceShowcaseSection from "@/components/home/sections/PerformanceShowcaseSection";
import type React from "react";

export default function HomePage() {
  return (
    <main className="overflow-x-hidden">
      <HeroSection />
      <div className={"relative w-full h-fit"}>
          <div
              className={
                  "ml-16 -mt-4 absolute z-10 size-8 bg-fd-background border border-fd-muted-foreground/20 rounded-lg"
              }
          />
          <div
              className={
                  "contents *:border-b *:border-fd-muted-foreground/20 *:absolute *:w-full *:top-0"
              }
          >
              <div className={"-mt-1"} />
              <div className={""} />
          </div>
          <ScalabilityShowcaseSection />
          <PolyglotShowcaseSection />
          <PerformanceShowcaseSection />
          <div className={`mb-96 *:mask-b-from-10% *:z-5 *:border-r *:right-0 *:border-fd-muted-foreground/20 *:absolute *:h-25`}>
              <div className={"mr-[7.85rem]"} />
              <div className={"mr-[8.15rem]"}/>
          </div>
      </div>

    </main>
  );
}
