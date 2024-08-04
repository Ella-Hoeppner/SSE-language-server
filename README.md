WIP Language server for the SSE text format

# to do
* move cursor to the beginning of a form with ctrl-a
  * when already at the start of a form, move to the start of the preceding sibling
  * when already at the start of a form and there are no preceeding siblings, move cursor to the start of the parent
  * when in between two forms (not touching either, but in whitespace between), move to the start of the first rather than the start of the parent
    * the problem is that `innermost_enclosing_path` identifies the parent in these cases
      * should `innermost_enclosing_path` consider the whitespace after a form as part of what that form contains? or have a different function for that?
* move cursor to the end of a form with ctrl-d
* fix crash when expanding selection with whole document selected
* formatting
  * optionally, this should happen automatically on save
* move selected form forward/backward in its parent with ctrl-w/s
* autoinsert closing encloser for any opening encloser
* highlighting of enclosers (and operators I guess)
  * rainbow
* extra highlight (box outline?) around enclosers
