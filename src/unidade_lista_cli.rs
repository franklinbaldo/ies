//! Caso de uso da caixa: verifica cobertura antes de publicar ou exportar dados.
//! `ies unidade listar-processos` lê a visão detalhada; vinda da R, a sessão
//! fica na D (contracts/cli/unidade-listar-processos-visao-detalhada.md).
use crate::caixa_detalhada::LeituraDetalhada;
use crate::colheita_do_controle::{Colheita, EnvioColhendo};
use crate::indice_local::IndiceLocal;
use crate::relogin_cli::{self, NotaDoRelogin};
use crate::transporte::envio_post::ler_tela_do_controle;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::{self, ErroVisaoDoControle, TelaAtual};
use crate::{arquivo_caixa, configuracao, dominio, saida};
use serde_json::json;
use std::path::Path;

pub(super) fn listar_processos_da_unidade(
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
    pagina: Option<usize>,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    // Cada tentativa relê a sessão num retrato coerente (#222) e colhe na
    // geração dele.
    let (leitura, nota) = relogin_cli::ler(
        &arquivos,
        unidade,
        ErroVisaoDoControle::sessao_encerrada,
        || {
            let indice = IndiceLocal::da_configuracao(&arquivos);
            let sessao = ReferenciasDaSessao::ler(
                &arquivos.url_controle,
                &arquivos.origem,
                &arquivos.cookie,
            )
            .map_err(ErroVisaoDoControle::Transporte)?;
            let colheita = Colheita::das_referencias(&indice, &sessao);
            let tela = ler_tela_do_controle(&sessao).map_err(ErroVisaoDoControle::Transporte)?;
            colheita.colher(&tela.html, Some(&tela.mime));
            let atual = TelaAtual {
                url: tela.url,
                html: tela.html,
                mime: tela.mime,
            };
            visao_do_controle::listar_na_visao_detalhada(
                &tela.instalacao,
                &atual,
                unidade,
                pagina,
                sessao.cookie(),
                &mut visao_do_controle::EnvioContado::novo(EnvioColhendo {
                    colheita: &colheita,
                }),
            )
        },
    )
    .map_err(relogin_cli::subir)?;
    mostrar_caixa_detalhada(&leitura, "web", json_mode, destino, nota.as_ref())?;
    Ok(())
}

/// A lista D: um array com **uma** lista `categoria: null` e `colunas_ausentes`.
pub(crate) fn json_da_detalhada(leitura: &LeituraDetalhada, origem: &str) -> serde_json::Value {
    let mut lista = saida::lista_json(&saida::ListaObservada {
        unidade: Some(&leitura.unidade),
        origem,
        categoria: None,
        total_declarado: Some(leitura.total_declarado),
        faixa_observada: leitura.faixa_observada,
        processos: &leitura.processos,
    });
    if let Some(objeto) = lista.as_object_mut() {
        objeto.insert(
            "colunas_ausentes".to_owned(),
            json!(leitura.colunas_ausentes()),
        );
    }
    serde_json::Value::Array(vec![lista])
}

/// D é modo de exibição da sessão, não categoria: uma lista, `categoria: null`.
pub(crate) fn mostrar_caixa_detalhada(
    leitura: &LeituraDetalhada,
    origem: &str,
    json_mode: bool,
    destino: Option<&Path>,
    nota: Option<&NotaDoRelogin>,
) -> Result<(), arquivo_caixa::ErroArquivoCaixa> {
    let mut saida = json_da_detalhada(leitura, origem);
    if let Some(caminho) = destino {
        // O arquivo guarda só a lista; `relogin` vai na linha de stdout.
        let bytes = arquivo_caixa::gravar(caminho, &saida)?;
        if json_mode {
            let mut valor = json!({
                "arquivo": "gravado",
                "bytes": bytes,
                "escopo": "caixa_detalhada_verificada"
            });
            relogin_cli::anotar_com_total(&mut valor, nota);
            println!("{valor}");
        } else {
            relogin_cli::avisar(json_mode, nota);
            println!("IES: caixa detalhada verificada gravada em arquivo privado.");
        }
        return Ok(());
    }
    if json_mode {
        relogin_cli::anotar_com_total(&mut saida, nota);
        println!(
            "{}",
            serde_json::to_string_pretty(&saida).expect("JSON serializável")
        );
    } else {
        relogin_cli::avisar(json_mode, nota);
        println!("{}", humano_da_detalhada(leitura));
    }
    Ok(())
}

fn humano_da_detalhada(leitura: &LeituraDetalhada) -> String {
    use std::fmt::Write as _;
    let faixa = leitura
        .faixa_observada
        .map_or_else(|| "nenhuma".to_owned(), |(i, f)| format!("{i} a {f}"));
    let mut texto = format!(
        "IES: visão detalhada da unidade {}: {} processo(s) observado(s); total declarado: {}; faixa observada: {faixa}.",
        leitura.unidade,
        leitura.processos.len(),
        leitura.total_declarado
    );
    for processo in &leitura.processos {
        let categoria = processo
            .categoria
            .map_or("categoria não observada", dominio::Categoria::rotulo);
        let _ = write!(
            texto,
            "\n{} ({}; {categoria})",
            processo.numero,
            processo.id.as_str()
        );
    }
    let ausentes = leitura.colunas_ausentes();
    if !ausentes.is_empty() {
        let _ = write!(
            texto,
            "\nIES: colunas não exibidas na visão detalhada: {}. Para vê-las, \
             `ies unidade colunas --definir todas` mostra a prévia de marcar todas as opções \
             desta instalação no Nível de detalhe (aplica com --confirmar); `ies unidade colunas` \
             mostra a configuração atual.",
            ausentes.join(", ")
        );
    }
    texto
}

#[cfg(test)]
#[path = "unidade_lista_cli/tests_faixa.rs"]
mod tests_faixa;
