import Aeneas
import RustVerify.Sandbox

open Aeneas.Std Result Error

theorem add_spec {a b : I32} (h₀ : I32.min ≤ ↑a + ↑b) (h₁ : ↑a + ↑b ≤ I32.max) :
  ∃ c, sandbox.add a b = ok c ∧ (↑c: ℤ) = ↑a + ↑b := by
  unfold sandbox.add
  apply I32.add_spec h₀ h₁

theorem fma_spec {a b c : I32}
  (h₀ : I32.min ≤ ↑a * ↑b)
  (h₁ : ↑a * ↑b ≤ I32.max)
  (h₂ : I32.min ≤ ↑a * ↑b + ↑c)
  (h₃ : ↑a * ↑b + ↑c ≤ I32.max) :
  ∃ d, sandbox.fma a b c = ok d ∧ (↑d: ℤ) = ↑a * ↑b + ↑c := by
  unfold sandbox.fma
  progress as ⟨x, xh⟩
  progress as ⟨y, yh⟩
  rw [yh, xh]

@[progress]
theorem UScalar.xor_spec {ty} (x y : UScalar ty) :
  ∃ z, toResult (x ^^^ y) = ok z ∧ z.val = (x ^^^ y).val ∧ z.bv = x.bv ^^^ y.bv := by
  simp [toResult]
  rfl

theorem Pcg64Si.next_u64_no_panic (self : sandbox.Pcg64Si) :
  ∃ res : U64 × sandbox.Pcg64Si, self.next_u64 = ok res := by
  unfold sandbox.Pcg64Si.next_u64
  progress*
