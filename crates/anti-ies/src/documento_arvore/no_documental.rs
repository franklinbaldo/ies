//! Projeção do nó documental na árvore mista.
use super::{Ciencia, Situacao, js};
use std::fmt::Write as _;

pub(super) fn no_de_documento(
    indice: usize,
    processo: &str,
    id: &str,
    numero: &str,
    tipo: &str,
    (versoes, lista_visivel, editor): (bool, bool, &str),
    (situacao, ciencia): (&Situacao, &Ciencia),
) -> String {
    let (icone, dica) = match situacao {
        Situacao::Disponivel => ("documento_interno", "Documento interno".to_owned()),
        Situacao::Cancelado { motivo } => (
            "documento_cancelado",
            format!("Documento Cancelado: {motivo}"),
        ),
    };
    let dica = js(&dica);
    let id = js(id);
    let processo = js(processo);
    let numero = js(numero);
    let rotulo = js(&format!("{tipo} {numero}"));
    let ciencia_acao = crate::documento_ciencia::barra(ciencia);
    let lista_ciencia =
        crate::documento_ciencia::acao_da_lista(indice + 9, &id, &processo, ciencia);
    let tela = format!(
        "controlador.php?acao=arvore_visualizar&id_procedimento={processo}&id_documento={id}&infra_hash={}",
        crate::protocolo_sintetico::INFRA_HASH
    );
    let cancelar = if matches!(situacao, Situacao::Disponivel) {
        format!(
            "<a href=\"controlador.php?acao=documento_cancelar&acao_origem=arvore_visualizar&id_procedimento={processo}&id_documento={id}&infra_hash={}\">Cancelar Documento</a>",
            crate::protocolo_sintetico::INFRA_HASH
        )
    } else {
        String::new()
    };
    let mut no = format!(
        r#"Nos[{indice}] = new infraArvoreNo("DOCUMENTO","{id}","{processo}","{tela}","ifrVisualizacao","{rotulo}","{dica}","svg/{icone}.svg","svg/{icone}.svg","svg/{icone}.svg",true,false,null,null,"noVisitado","{numero}");
"#
    );
    if versoes {
        let hash = "e".repeat(64);
        let link_lista = if lista_visivel && matches!(situacao, Situacao::Disponivel) {
            format!(
                "<a href=\"controlador.php?acao=documento_versao_listar&id_procedimento={processo}&id_documento={id}&infra_hash={hash}\">Versões</a>"
            )
        } else {
            String::new()
        };
        let hash_corpo = "f".repeat(64);
        let src = match situacao {
            Situacao::Disponivel => format!(
                "controlador.php?acao=documento_visualizar&id_documento={id}&infra_hash={hash_corpo}"
            ),
            Situacao::Cancelado { .. } => String::new(),
        };
        write!(
            no,
            "Nos[{indice}].acoes = '{link_lista}{editor}{cancelar}{ciencia_acao}';\nNos[{indice}].src = '{src}';\n"
        )
        .expect("formatação de String");
    } else {
        write!(
            no,
            "Nos[{indice}].acoes = '{cancelar}{ciencia_acao}';\nNos[{indice}].src = '';\n"
        )
        .expect("formatação de String");
    }
    writeln!(no, "Nos[{indice}].html = '';").expect("formatação de String");
    no.push_str(&lista_ciencia);
    no
}
