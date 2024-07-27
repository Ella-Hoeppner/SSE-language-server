WIP Language server for the SSE text format

### feature goals
* structural editing hotkeys, similar to paredit
  * highlight current form, or if a form is already highlighted, form that encloses current form
  * move a form (highlighted, or if there is no highlight, simply whichever the mouse is inside) forward/backward/in/out of current enclosing form
  * what keys to use for this?

* autoinsert closing encloser for any opening encloser
* highlighting of enclosers (and operators I guess)
  * rainbow
* autoformat on save (toggleable)
