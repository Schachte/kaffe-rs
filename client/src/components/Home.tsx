import "./styles/base.css";
import "./styles/second.css";

import { useState } from "react";

const fn = (click: number) => {
  return <div className="styles_test">Button clicked: {click} times</div>;
};

const Home = () => {
  const [click, setClick] = useState(0);

  return (
    <div>
      {fn(click)}
      <button onClick={() => setClick(click + 1)}>YAY</button>
      <div className="second_file_test">test</div>
    </div>
  );
};

export default Home;
