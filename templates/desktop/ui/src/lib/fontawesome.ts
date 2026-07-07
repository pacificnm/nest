import { config, library } from "@fortawesome/fontawesome-svg-core";
import {
  faBars,
  faCalendar,
  faChevronDown,
  faChevronLeft,
  faChevronRight,
  faChevronUp,
  faCircleCheck,
  faCircleExclamation,
  faCircleInfo,
  faDoorOpen,
  faGear,
  faMagnifyingGlass,
  faPlus,
  faRotateRight,
  faTrash,
  faTriangleExclamation,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";

import "@fortawesome/fontawesome-svg-core/styles.css";

// We add CSS manually via the imported stylesheet; disable auto-injection to
// avoid the flash of oversized icons before the app stylesheet loads.
config.autoAddCss = false;

library.add(
  faBars,
  faCalendar,
  faChevronDown,
  faChevronLeft,
  faChevronRight,
  faChevronUp,
  faCircleCheck,
  faCircleExclamation,
  faCircleInfo,
  faDoorOpen,
  faGear,
  faMagnifyingGlass,
  faPlus,
  faRotateRight,
  faTrash,
  faTriangleExclamation,
  faXmark,
);

export {
  faBars,
  faCalendar,
  faChevronDown,
  faChevronLeft,
  faChevronRight,
  faChevronUp,
  faCircleCheck,
  faCircleExclamation,
  faCircleInfo,
  faDoorOpen,
  faGear,
  faMagnifyingGlass,
  faPlus,
  faRotateRight,
  faTrash,
  faTriangleExclamation,
  faXmark,
};
