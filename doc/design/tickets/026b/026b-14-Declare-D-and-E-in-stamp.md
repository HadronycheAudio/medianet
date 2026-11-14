# 026b-14 — Declare D and E in stamp
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-03, 026a-11, 026a-12

## Context

Audio edge: `emit(p) = T(p) + A`, `submit(p) = emit(p) − D`; D is the device's declared plumbing delay. D is the sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter (C-170); the edge's conversion delay is part of it (C-134). D components: `AudioOut::fixed_delay_ns()` (converter and interface), the fixed 2:1 node's whole-sample delay, the rate actuator's `group_delay_frames()` at 48 kHz (C-80, C-189), and buffering.

E is declared per device with the common definition (settled, C-73): **the 99.9th percentile of absolute emission error over a one-hour certification run, with the maximum reported beside it**. This closes the earlier "bound or percentile" question. Only the commissioned certification state gives a numeric E; uncertified and self-monitored sinks declare E unknown (C-140) and are refused with RequiredEExceeded by groups that set a required E (C-103). The certificate is stored beside the collected stamps in the deployment's data directory (C-171). D measurement and certification are commissioning measurements: taken by the commissioning tool inside an open commissioning window they are valid commissioning results, each recorded with the window it was taken in (C-178); only ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while a window is open. The earlier reading that the run must avoid any window is withdrawn. Cross-sink skew is judged against the sum of the two sinks' E values over 1 hour and across restarts (C-79, task 027).

Epic criterion 026-AC-10 (Edge figures declared): the device's **D (with method)** and **E** are in its stamp; contributes to AC-29 (cross-sink skew, verified in task 027).

Still open:
- The value of the device's E: produced here, confirmed by task 027 (L3); falsifier: measured skew exceeds the declared sum.
- The list of allowed methods for "D with method": owner epic 018; this ticket records the method it used in plain words until that list exists.

## Work

1. Measure D on the bench: a click at known position p, observe the analog output with a scope against a reference (e.g. a Linux host's PPS/GPIO at T(p)); D_measured = output time − submit-scheduled time components. Record the method and the commissioning window it was taken in, if any (C-178).
2. Run a one-hour commissioned certification of absolute emission error (C-73, C-140); when the tool drives test traffic for it, run it inside a declared commissioning window (C-107) and record that window with the result (C-178); compute the 99.9th percentile and the maximum; store the certificate beside the collected stamps (C-171). The certification record carries both stamps: the regime stamp of the device and that of the holder it followed (C-191).
3. Set the D, D method and E (`Certified { p999_ns, max_ns }`) fields in the firmware profile; before step 2 completes, the firmware declares E `Unknown` (C-103). Confirm the stamp recorded by the hosted collector shows them.
4. Store the measurements in `doc/design/reports/` (C-130).

## Acceptance criteria

1. D value and method recorded `[measured]`, with the commissioning window it was taken in, if any (C-178); datasheet-only components tagged `[memory]`.
2. E declared as the 99.9th percentile over a one-hour commissioned run with the maximum beside it (C-73, C-140), with the measurement log and the certificate stored per C-171; the record holds the regime stamps of the device and of the holder it followed (C-191) and names the commissioning window the run was taken in, if any (C-178).
3. The collector's record for the board contains D, D method and E equal to the declared values.
4. A firmware build without a certified E stamps E as unknown (C-103).

## Definition of done

- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope

- Cross-sink skew (AC-29, C-79) and the confirmation of E: task 027.
- The device's restart spread of D (C-191): 026b-20.
