# Gemini Code Review Style Guide

This document outlines the basic style rules and review policy for code reviewed using Gemini.

## 🔍 What to Review

Only leave review comments on the following issues:

1. **Bugs or Bug-Prone Logic**  
   - Obvious implementation mistakes (e.g., incorrect conditions, off-by-one errors)
   - Code that may cause runtime errors or incorrect behavior

2. **Incorrect or Mismatched Comments**  
   - Comments that do not match the actual logic of the code
   - Misleading or outdated comments

3. **Language Consistency**  
   - Japanese comments or any Japanese text mixed into the code **must be pointed out**
   - All comments should be written in **English**

4. **Debug Code Left Behind**  
   - Any remaining debug logs, print statements, or comments used during development must be flagged for removal

5. **Clear Best Practices**  
   - If there is an evident and significant improvement to code quality by following a well-known best practice, suggest the improvement

## 🚫 What Not to Review

- **Lack of Comments**: Do not request additional comments even if the logic seems difficult to read
- **Code Style or Formatting**: Do not point out minor stylistic issues unless they cause bugs or severely reduce clarity

## ✅ Goal

The goal of the review is to ensure correctness, consistency, and maintainability without overloading developers with stylistic or preference-based suggestions.