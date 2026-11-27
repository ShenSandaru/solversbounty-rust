export const JOURNEY_STORAGE_KEY = "sb-onboarding-journey";
export const JOURNEY_CHANGE_EVENT = "sb-onboarding-journey-change";

export type OnboardingJourney = "solver" | "creator";

export function readOnboardingJourney(): OnboardingJourney | null {
  if (typeof window === "undefined") {
    return null;
  }
  const value = window.localStorage.getItem(JOURNEY_STORAGE_KEY);
  return value === "solver" || value === "creator" ? value : null;
}

export function writeOnboardingJourney(journey: OnboardingJourney): void {
  window.localStorage.setItem(JOURNEY_STORAGE_KEY, journey);
  notifyJourneyChange();
}

export function clearOnboardingJourney(): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.removeItem(JOURNEY_STORAGE_KEY);
  notifyJourneyChange();
}

export function subscribeOnboardingJourney(onStoreChange: () => void): () => void {
  if (typeof window === "undefined") {
    return () => undefined;
  }
  window.addEventListener("storage", onStoreChange);
  window.addEventListener(JOURNEY_CHANGE_EVENT, onStoreChange);
  return () => {
    window.removeEventListener("storage", onStoreChange);
    window.removeEventListener(JOURNEY_CHANGE_EVENT, onStoreChange);
  };
}

function notifyJourneyChange(): void {
  if (typeof window === "undefined") {
    return;
  }
  window.dispatchEvent(new Event(JOURNEY_CHANGE_EVENT));
}
