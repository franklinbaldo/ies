//! `ies documento buscar TERMO [--pagina N]` (contracts/cli/documento-buscar.md):
//! valida o termo (saída 2 antes de ler a configuração), lê os arquivos
//! privados da sessão e publica a resposta observada da pesquisa Web em
//! documentos, pelas mesmas peças de `ies processo buscar`.
use crate::colheita_do_controle::Colheita;
use crate::indice_local::IndiceLocal;
use crate::pesquisa_web::documentos::{EmDocumentos, RespostaDeDocumentos};
use crate::pesquisa_web::fluxo::{BuscaRealizada, PedidoDeBusca, SessaoDaBusca, buscar_colhendo};
use crate::visao_do_controle::EnvioHttps;
use crate::{
    Cli, Documento, DocumentoAcao, Domain, configuracao, erro_fonte, saida_erro, transporte,
};
use serde_json::{Value, json};
use std::process::ExitCode;

/// Termo da pesquisa em documentos: texto opaco do campo Protocolo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TermoDeDocumentoInvalido {
    Vazio,
    Controle,
}

impl TermoDeDocumentoInvalido {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Vazio => "termo_de_documento_vazio",
            Self::Controle => "termo_de_documento_contem_controle",
        }
    }
}

/// Controles recusados antes de aparar (trim os apagaria); sem normalizar.
pub(crate) fn validar_termo(termo: &str) -> Result<&str, TermoDeDocumentoInvalido> {
    if termo.chars().any(char::is_control) {
        return Err(TermoDeDocumentoInvalido::Controle);
    }
    let termo = termo.trim();
    if termo.is_empty() {
        return Err(TermoDeDocumentoInvalido::Vazio);
    }
    Ok(termo)
}

/// Argumentos de leitura de `ies documento DOC` que não combinam com `buscar`.
fn opcoes_de_leitura_presentes(comando: &Documento) -> bool {
    !comando.numeros.is_empty()
        || comando.processo.is_some()
        || comando.versao.is_some()
        || comando.sem_copia
        || comando.html_privado.is_some()
        || comando.saida.is_some()
        || comando.ocultar_inacessiveis
        || comando.com_propriedades
}

fn erro_de_uso(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}

/// O JSON da resposta observada; nunca HTML, links, cookies ou `infra_hash`.
pub(crate) fn json_da_busca(termo: &str, busca: &BuscaRealizada<RespostaDeDocumentos>) -> Value {
    let mut valor = json!({
        "termo": termo,
        "unidade": busca.unidade,
        "origem": "web",
        "requisicoes": busca.requisicoes,
    });
    let extra = match &busca.resposta {
        // O 302 só traz os ids internos: número SEI e NUP não são inventados.
        RespostaDeDocumentos::DocumentoUnico {
            documento,
            processo,
        } => json!({
            "resultado": "documento_unico",
            "documento": {"id": documento},
            "processo": {"id": processo.as_str()},
        }),
        RespostaDeDocumentos::Lista(lista) => json!({
            "resultado": "lista",
            "total_declarado": lista.total_declarado,
            "faixa_observada": {"inicio": lista.faixa.inicio, "fim": lista.faixa.fim},
            "linhas_ausentes_da_faixa": lista.linhas_ausentes_da_faixa,
            "documentos": lista
                .documentos
                .iter()
                .map(|d| json!({
                    "numero_sei": d.numero_sei,
                    "rotulo": d.rotulo,
                    "processo": d.processo.as_str(),
                    "unidade": d.unidade,
                    "inclusao": d.inclusao,
                    "anexo": d.anexo,
                }))
                .collect::<Vec<_>>(),
        }),
        // Resposta observada para esta conta/unidade, não inexistência.
        RespostaDeDocumentos::SemResultado => json!({
            "resultado": "sem_resultado",
            "total_declarado": 0,
            "faixa_observada": null,
            "documentos": [],
        }),
    };
    if let (Some(objeto), Value::Object(campos)) = (valor.as_object_mut(), extra) {
        objeto.extend(campos);
    }
    valor
}

fn humano(termo: &str, busca: &BuscaRealizada<RespostaDeDocumentos>) -> String {
    let detalhe = match &busca.resposta {
        RespostaDeDocumentos::DocumentoUnico {
            documento,
            processo,
        } => format!(
            "um documento (id interno {documento}, processo de id interno {})",
            processo.as_str()
        ),
        RespostaDeDocumentos::Lista(lista) => {
            let numeros: Vec<String> = lista
                .documentos
                .iter()
                .map(|d| format!("{} ({}, {})", d.numero_sei, d.rotulo, d.processo.as_str()))
                .collect();
            let ausentes = match lista.linhas_ausentes_da_faixa {
                0 => String::new(),
                n => format!(", {n} sem linha na tabela"),
            };
            format!(
                "{} de {} declarado(s), faixa {} a {}{ausentes}: {}",
                numeros.len(),
                lista.total_declarado,
                lista.faixa.inicio,
                lista.faixa.fim,
                numeros.join("; ")
            )
        }
        RespostaDeDocumentos::SemResultado => {
            "nenhum resultado para esta conta e unidade (não prova inexistência)".to_owned()
        }
    };
    format!(
        "IES: pesquisa em documentos de {termo:?} na unidade {}: {detalhe} ({} requisições).",
        busca.unidade, busca.requisicoes
    )
}

fn buscar(cli: &Cli, comando: &Documento, termo: &str, pagina: Option<u32>) -> ExitCode {
    if opcoes_de_leitura_presentes(comando) {
        return erro_de_uso(
            cli.json,
            "opcoes_incompativeis_com_busca",
            "opções de leitura do documento não combinam com buscar",
        );
    }
    let termo = match validar_termo(termo) {
        Ok(termo) => termo,
        Err(erro) => return erro_de_uso(cli.json, erro.subcodigo(), "termo de pesquisa inválido"),
    };
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.subcodigo(), erro.mensagem()),
    };
    let lidos =
        transporte::sessao::ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem);
    let referencias = match lidos {
        Ok(referencias) => referencias,
        Err(falha) => return erro_fonte(cli.json, falha.subcodigo(), falha.mensagem()),
    };
    let sessao = SessaoDaBusca {
        url_do_controle: referencias.url_do_controle(),
        origem: referencias.origem(),
        cookie_file: &arquivos.cookie,
    };
    let pedido = PedidoDeBusca {
        termo,
        pagina,
        unidade: cli.unidade.as_deref(),
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let colheita = Colheita::da_sessao(&indice, &arquivos);
    match buscar_colhendo::<EmDocumentos, _>(&sessao, &pedido, &colheita, &mut EnvioHttps) {
        Ok(busca) => {
            if cli.json {
                let valor = json_da_busca(termo, &busca);
                println!("{}", serde_json::to_string_pretty(&valor).unwrap());
            } else {
                println!("{}", humano(termo, &busca));
            }
            ExitCode::SUCCESS
        }
        Err(erro) => saida_erro::erro_fonte_http(
            cli.json,
            erro.subcodigo(),
            erro.mensagem(),
            erro.status_http(),
        ),
    }
}

/// `Some` só para `ies documento buscar`.
pub(crate) fn tentar_buscar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(comando) = &cli.command else {
        return None;
    };
    let Some(DocumentoAcao::Buscar { termo, pagina }) = &comando.acao else {
        return None;
    };
    Some(buscar(cli, comando, termo, *pagina))
}

#[cfg(test)]
#[path = "documento_buscar_cli/tests.rs"]
mod tests;
