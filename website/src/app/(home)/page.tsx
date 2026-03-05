"use client";

import HeroSection from "@/components/home/sections/HeroSection";
import IntegrationsSection from "@/components/home/sections/IntegrationsSection";

export default function HomePage() {
  return (
    <main className="overflow-x-hidden">
      <HeroSection />
      <IntegrationsSection />
    </main>
  );
}
