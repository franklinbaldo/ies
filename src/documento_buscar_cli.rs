//! `ies documento buscar TERMO [--pagina N]` (contracts/cli/documento-buscar.md):
//! valida o termo (saída 2 antes de ler a configuração), lê os arquivos
//! privados da sessão e publica a resposta observada da pesquisa Web em
//! documentos, pelas mesmas peças de `ies processo buscar`.
use crate::colheita_do_controle::Colheita;
use crate::indice_local::IndiceLocal;
use crate::pesquisa_web::documentos::{EmDocumentos, RespostaDeDocumentos};
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::filtros::{FiltrosDaPesquisa, data_valida};
use crate::pesquisa_web::fluxo::{
    BuscaRealizada, PedidoDeBusca, SessaoDaBusca, buscar_colhendo_filtrado,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{
    Cli, Documento, DocumentoAcao, Domain, configuracao, erro_fonte, relogin_cli, transporte,
};
use serde_json::{Value, json};
use std::process::ExitCode;

/// Termo da pesquisa em documentos: texto opaco do campo Protocolo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum TermoDeDocumentoInvalido {
    Vazio,
    Controle,
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
        || comando.html_privado.is_some()
        || comando.saida.is_some()
        || comando.ocultar_inacessiveis
        || comando.com_propriedades
}

fn erro_de_uso(json_mode: bool, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem}");
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

fn buscar(
    cli: &Cli,
    comando: &Documento,
    termo: &str,
    pagina: Option<u32>,
    filtros: FiltrosDaPesquisa<'_>,
) -> ExitCode {
    if filtros.de.is_some_and(|v| !data_valida(v)) || filtros.ate.is_some_and(|v| !data_valida(v)) {
        return erro_de_uso(
            cli.json,
            "data deve estar em DD/MM/AAAA e existir no calendário",
        );
    }
    if let (Some(de), Some(ate)) = (filtros.de, filtros.ate) {
        let inicio = format!("{}{}{}", &de[6..10], &de[3..5], &de[0..2]);
        let fim = format!("{}{}{}", &ate[6..10], &ate[3..5], &ate[0..2]);
        if inicio > fim {
            return erro_de_uso(cli.json, "data inicial posterior à final");
        }
    }
    if opcoes_de_leitura_presentes(comando) {
        return erro_de_uso(
            cli.json,
            "opções de leitura do documento não combinam com buscar",
        );
    }
    let Ok(termo) = validar_termo(termo) else {
        return erro_de_uso(cli.json, "termo de pesquisa inválido");
    };
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.mensagem()),
    };
    let pedido = PedidoDeBusca {
        termo,
        pagina,
        unidade: cli.unidade.as_deref(),
    };
    // Cada tentativa relê a sessão e o índice: depois do relogin, a nova.
    let lido = relogin_cli::ler(&arquivos, cli.unidade.as_deref(), || {
        let referencias = transporte::sessao::ReferenciasDaSessao::ler(
            &arquivos.url_controle,
            &arquivos.origem,
            &arquivos.cookie,
        )
        .map_err(ErroBuscaDeProcesso::Transporte)?;
        let sessao = SessaoDaBusca {
            url_do_controle: referencias.url_do_controle(),
            origem: referencias.origem(),
            cookie: referencias.cookie(),
        };
        let indice = IndiceLocal::da_configuracao(&arquivos);
        let colheita = Colheita::das_referencias(&indice, &referencias);
        buscar_colhendo_filtrado::<EmDocumentos, _>(
            &sessao,
            &pedido,
            filtros,
            &colheita,
            &mut EnvioHttps,
        )
    });
    match lido {
        Ok((mut busca, nota)) => {
            busca.requisicoes = relogin_cli::somar(busca.requisicoes, nota.as_ref());
            if cli.json {
                let mut valor = json_da_busca(termo, &busca);
                relogin_cli::anotar(&mut valor, nota.as_ref());
                if let Err(erro) = crate::saida::imprimir_json(&valor) {
                    eprintln!("IES: falha ao emitir JSON: {erro}");
                    return std::process::ExitCode::from(1);
                }
            } else {
                relogin_cli::avisar(cli.json, nota.as_ref());
                println!("{}", humano(termo, &busca));
            }
            ExitCode::SUCCESS
        }
        Err(falha) => relogin_cli::publicar_falha(cli.json, &falha),
    }
}

/// `Some` só para `ies documento buscar`.
pub(crate) fn tentar_buscar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(comando) = &cli.command else {
        return None;
    };
    let Some(DocumentoAcao::Buscar {
        termo,
        pagina,
        tipo,
        assunto,
        de,
        ate,
    }) = &comando.acao
    else {
        return None;
    };
    Some(buscar(
        cli,
        comando,
        termo,
        *pagina,
        FiltrosDaPesquisa {
            tipo: tipo.as_deref(),
            assunto: assunto.as_deref(),
            de: de.as_deref(),
            ate: ate.as_deref(),
        },
    ))
}

#[cfg(test)]
#[path = "documento_buscar_cli/tests.rs"]
mod tests;
