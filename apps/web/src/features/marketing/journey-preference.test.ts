import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  JOURNEY_CHANGE_EVENT,
  JOURNEY_STORAGE_KEY,
  clearOnboardingJourney,
  readOnboardingJourney,
  writeOnboardingJourney,
} from "./journey-preference";

describe("onboarding journey preference", () => {
  const store = new Map<string, string>();
  const listeners = new Map<string, Set<EventListener>>();

  beforeEach(() => {
    store.clear();
    listeners.clear();
    vi.stubGlobal("window", {
      localStorage: {
        getItem: (key: string) => store.get(key) ?? null,
        setItem: (key: string, value: string) => {
          store.set(key, value);
        },
        removeItem: (key: string) => {
          store.delete(key);
        },
      },
      addEventListener: (type: string, listener: EventListener) => {
        const set = listeners.get(type) ?? new Set();
        set.add(listener);
        listeners.set(type, set);
      },
      removeEventListener: (type: string, listener: EventListener) => {
        listeners.get(type)?.delete(listener);
      },
      dispatchEvent: (event: Event) => {
        listeners.get(event.type)?.forEach((listener) => listener(event));
        return true;
      },
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("returns null when nothing is stored", () => {
    expect(readOnboardingJourney()).toBeNull();
  });

  it("stores solver or creator for navigation only", () => {
    writeOnboardingJourney("solver");
    expect(readOnboardingJourney()).toBe("solver");
    writeOnboardingJourney("creator");
    expect(readOnboardingJourney()).toBe("creator");
    expect(store.get(JOURNEY_STORAGE_KEY)).toBe("creator");
  });

  it("ignores unknown stored values", () => {
    store.set(JOURNEY_STORAGE_KEY, "admin");
    expect(readOnboardingJourney()).toBeNull();
  });

  it("clears the preference and notifies same-tab listeners", () => {
    const onChange = vi.fn();
    window.addEventListener(JOURNEY_CHANGE_EVENT, onChange);
    writeOnboardingJourney("solver");
    clearOnboardingJourney();
    expect(readOnboardingJourney()).toBeNull();
    expect(onChange).toHaveBeenCalled();
  });
});
