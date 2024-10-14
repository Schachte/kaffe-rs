import { Routes, Route } from "react-router-dom";
import Home from "./Home";
import Contact from "./Contact";
import NotFound from "./NotFound";

export default function Routing() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/contact" element={<Contact />} />
      <Route path="*" element={<NotFound />} />
    </Routes>
  );
}
