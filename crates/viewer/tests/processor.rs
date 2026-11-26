// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the geometry processor, which runs without a window.

use bevy::mesh::{Mesh, VertexAttributeValues};
use mucad_viewer::{
    Config,
    processor::{ProcessorInterface, ProcessorRequest, ProcessorResponse},
};

/// A processor, which finds the standard library of this repository.
fn processor() -> ProcessorInterface {
    let processor = ProcessorInterface::run();
    processor
        .send_request(ProcessorRequest::Initialize {
            config: Config {
                // The standard library of this repository (relative to `crates/viewer`).
                search_paths: vec!["../std/lib".into()],
                ..Default::default()
            },
        })
        .expect("No error");
    processor
}

/// The responses for the compilation of `source` until the model instances are spawned.
fn responses(source: &str) -> Vec<ProcessorResponse> {
    let processor = processor();
    processor
        .send_request(ProcessorRequest::CompileSource {
            path: None,
            name: None,
            source: source.into(),
        })
        .expect("No error");

    let mut responses = Vec::new();
    while let Ok(response) = processor
        .response_receiver
        .recv_timeout(std::time::Duration::from_secs(30))
    {
        let spawned = matches!(response, ProcessorResponse::SpawnModelInstances(_));
        responses.push(response);
        if spawned {
            break;
        }
    }
    responses
}

/// Each workpiece with geometry gets a model info with its meshes.
#[test]
fn process_workpieces() {
    let responses = responses("Circle(radius = 1mm);\nRect(2mm).translate(x = 5mm);");

    let meshes = responses
        .iter()
        .filter(|response| matches!(response, ProcessorResponse::NewMeshAsset(..)))
        .count();
    let model_infos: Vec<_> = responses
        .iter()
        .filter_map(|response| match response {
            ProcessorResponse::NewModelInfo(_, info) => Some(info),
            _ => None,
        })
        .collect();

    assert_eq!(model_infos.len(), 2);
    assert_eq!(meshes, 2);
    assert!(model_infos.iter().all(|info| info.mesh_uuids.len() == 1));
    assert!(matches!(
        responses.last(),
        Some(ProcessorResponse::SpawnModelInstances(uuids)) if uuids.len() == 2
    ));
}

/// A source with errors spawns no models.
#[test]
fn process_errors() {
    let processor = processor();
    processor
        .send_request(ProcessorRequest::CompileSource {
            path: None,
            name: None,
            source: "Unknown();".into(),
        })
        .expect("No error");

    let responses: Vec<_> = processor
        .response_receiver
        .iter()
        .take_while(|response| {
            !matches!(
                response,
                ProcessorResponse::StateChanged(mucad_viewer::processor::ProcessingState::Error)
            )
        })
        .collect();
    assert!(
        !responses
            .iter()
            .any(|response| matches!(response, ProcessorResponse::NewModelInfo(..)))
    );
}

/// A part's color attribute is copied to every vertex of its generated mesh.
#[test]
fn part_color_is_written_to_mesh_vertices() {
    let responses = responses("#[color = \"red\"]\nCircle(radius = 1mm);");

    let mesh = responses
        .iter()
        .find_map(|response| match response {
            ProcessorResponse::NewMeshAsset(_, mesh, _) => Some(mesh),
            _ => None,
        })
        .expect("colored part should generate a mesh");

    let vertex_count = mesh.count_vertices();
    match mesh
        .attribute(Mesh::ATTRIBUTE_COLOR)
        .expect("colored mesh should have a vertex color attribute")
    {
        VertexAttributeValues::Float32x4(colors) => {
            assert_eq!(colors.len(), vertex_count);
            assert!(!colors.is_empty());
            assert!(colors.iter().all(|color| *color == [1.0, 0.0, 0.0, 1.0]));
        }
        other => panic!("unexpected vertex color format: {other:?}"),
    }
}

/// A union keeps different part colors on separate meshes.
#[test]
fn union_preserves_part_colors_on_split_meshes() {
    let responses = responses(
        r#"{
            #[color = "red"] Circle(radius = 2mm);
            #[color = "blue"] Circle(radius = 1mm);
        }.__mu::ops::union();"#,
    );

    let meshes: Vec<_> = responses
        .iter()
        .filter_map(|response| match response {
            ProcessorResponse::NewMeshAsset(_, mesh, _) => Some(mesh),
            _ => None,
        })
        .collect();
    assert_eq!(meshes.len(), 2);

    let colors: Vec<_> = meshes
        .iter()
        .map(|mesh| {
            match mesh
                .attribute(Mesh::ATTRIBUTE_COLOR)
                .expect("vertex colors")
            {
                VertexAttributeValues::Float32x4(colors) => {
                    assert_eq!(colors.len(), mesh.count_vertices());
                    colors[0]
                }
                other => panic!("unexpected vertex color format: {other:?}"),
            }
        })
        .collect();
    assert!(colors.contains(&[1.0, 0.0, 0.0, 1.0]));
    assert!(colors.contains(&[0.0, 0.0, 1.0, 1.0]));
}
