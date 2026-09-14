//! Final-output-trained query substitution between contextual language reads.
pub mod completion;
pub mod completion_data;
#[cfg(test)]
mod completion_diagnostic;
pub mod completion_learning;
#[cfg(test)]
mod completion_report;
pub mod data;
pub mod depth_data;
#[cfg(test)]
mod depth_report;
pub mod joint_learning;
#[cfg(test)]
mod joint_report;
pub mod learning;
#[cfg(test)]
mod report;
pub mod runtime;
pub mod schedule_data;
pub mod schedule_learning;
#[cfg(test)]
mod schedule_report;
pub mod scheduling;

pub mod span;
pub mod span_boundary;
pub mod span_data;
pub mod span_learning;
#[cfg(test)]
mod span_report;

pub mod phrase;
pub mod phrase_data;
#[cfg(test)]
mod phrase_report;

pub mod occurrence;
pub mod occurrence_data;
pub mod occurrence_learning;
#[cfg(test)]
mod occurrence_qualification;
#[cfg(test)]
mod occurrence_report;

#[cfg(test)]
mod occurrence_collision;

pub mod correspondence;
#[cfg(test)]
mod correspondence_diagnostic;
pub mod correspondence_learning;
#[cfg(test)]
mod correspondence_report;

#[cfg(test)]
mod phrase_order_report;

#[cfg(test)]
mod lexical_role_report;

pub mod occurrence_role;
#[cfg(test)]
mod occurrence_role_report;

#[cfg(test)]
mod role_transfer_report;

pub mod query_participation;
#[cfg(test)]
mod query_participation_report;

#[cfg(test)]
mod styled_role_report;

#[cfg(test)]
mod neighbor_transfer_report;
