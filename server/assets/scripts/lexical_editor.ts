import {
    createEditor,
    $getRoot,
    $getSelection,
    $createParagraphNode,
    $createTextNode,
} from "../vendored/lexical/lexical.js";
import lexical from "../vendored/lexical/lexical.js";
import "../styles/lexical.css";

import "../vendored/lexical/selection.js";
import "../vendored/lexical/utils.js";
import "../vendored/lexical/html.js";
import "../vendored/lexical/clipboard.js";
import { LinkNode, AutoLinkNode } from "../vendored/lexical/link.js";
import { registerRichText, QuoteNode, HeadingNode } from "../vendored/lexical/rich_text.js";

const exampleTheme = {
    ltr: "ltr",
    rtl: "rtl",
    placeholder: "editor-placeholder",
    paragraph: "editor-paragraph",
    quote: "editor-quote",
    heading: {
      h1: "editor-heading-h1",
      h2: "editor-heading-h2",
      h3: "editor-heading-h3",
      h4: "editor-heading-h4",
      h5: "editor-heading-h5"
    },
    list: {
      nested: {
        listitem: "editor-nested-listitem"
      },
      ol: "editor-list-ol",
      ul: "editor-list-ul",
      listitem: "editor-listitem"
    },
    image: "editor-image",
    link: "editor-link",
    text: {
      bold: "editor-text-bold",
      italic: "editor-text-italic",
      overflowed: "editor-text-overflowed",
      hashtag: "editor-text-hashtag",
      underline: "editor-text-underline",
      strikethrough: "editor-text-strikethrough",
      underlineStrikethrough: "editor-text-underlineStrikethrough",
      code: "editor-text-code"
    },
    code: "editor-code",
    codeHighlight: {
      atrule: "editor-tokenAttr",
      attr: "editor-tokenAttr",
      boolean: "editor-tokenProperty",
      builtin: "editor-tokenSelector",
      cdata: "editor-tokenComment",
      char: "editor-tokenSelector",
      class: "editor-tokenFunction",
      "class-name": "editor-tokenFunction",
      comment: "editor-tokenComment",
      constant: "editor-tokenProperty",
      deleted: "editor-tokenProperty",
      doctype: "editor-tokenComment",
      entity: "editor-tokenOperator",
      function: "editor-tokenFunction",
      important: "editor-tokenVariable",
      inserted: "editor-tokenSelector",
      keyword: "editor-tokenAttr",
      namespace: "editor-tokenVariable",
      number: "editor-tokenProperty",
      operator: "editor-tokenOperator",
      prolog: "editor-tokenComment",
      property: "editor-tokenProperty",
      punctuation: "editor-tokenPunctuation",
      regex: "editor-tokenVariable",
      selector: "editor-tokenSelector",
      string: "editor-tokenSelector",
      symbol: "editor-tokenProperty",
      tag: "editor-tokenProperty",
      url: "editor-tokenOperator",
      variable: "editor-tokenVariable"
    }
  };
  

  

const editorConfig = {
    // The editor theme
    theme: exampleTheme,
    // Handling of errors during update
    onError(error) {
      throw error;
    },
    // Any custom nodes go here
    nodes: [
      HeadingNode,
      //ListNode,
      //ListItemNode,
      QuoteNode,
      //CodeNode,
      //CodeHighlightNode,
      //TableNode,
      //TableCellNode,
      //TableRowNode,
      //AutoLinkNode,
      //LinkNode
    ]
  };

const editor = createEditor(editorConfig);
registerRichText(editor);

const contentEditableElement = document.getElementById("editor");

editor.setRootElement(contentEditableElement);

editor.update(() => {
    const root = $getRoot(); // Get the RootNode from the EditorState
    const selection = $getSelection(); // Get the selection from the EditorState
    const paragraphNode = $createParagraphNode(); // Create a new ParagraphNode
    const textNode = $createTextNode("Hello world from ESM"); // Create a new TextNode
    paragraphNode.append(textNode); // Append the text node to the paragraph
    root.append(paragraphNode); // Finally, append the paragraph to the root
});