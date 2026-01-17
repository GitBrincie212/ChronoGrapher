"use client";

import HeroSection from "@/components/home/sections/HeroSection";
import ShowcaseSection from "@/components/home/sections/ShowcaseSection";

export default function HomePage() {
  return (
    <main className="overflow-x-hidden">
      <HeroSection />
      <ShowcaseSection />
    </main>
  );
}
