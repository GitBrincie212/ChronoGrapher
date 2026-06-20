"use client";

import HeroSection from "@/components/home/sections/HeroSection";
import ScalabilityShowcaseSection from "@/components/home/sections/ScalabilityShowcaseSection";

export default function HomePage() {
  return (
    <main className="overflow-x-hidden">
      <HeroSection />
      <ScalabilityShowcaseSection />
    </main>
  );
}
