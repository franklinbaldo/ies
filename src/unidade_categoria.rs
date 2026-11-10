//! `ies unidade recebidos` e `ies unidade gerados`: uma lista independente
//! da caixa, a página observada com total e faixa; `--pagina N` acrescenta
//! no máximo um POST (`pagina_web`). Na visão D, troca para R e fica nela.
mod pagina_web;

use crate::paginas::Leitura;
use crate::relogin_cli::{self, NotaDoRelogin};
use crate::{arquivo_caixa, configuracao, erro_fonte, paginas};
use serde_json::{Value, json};
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CategoriaLista {
    Recebidos,
    Gerados,
}

impl CategoriaLista {
    pub(crate) fn categoria(self) -> crate::dominio::Categoria {
        match self {
            Self::Recebidos => crate::dominio::Categoria::Recebidos,
            Self::Gerados => crate::dominio::Categoria::Gerados,
        }
    }

    pub(crate) fn rotulo(self) -> &'static str {
        self.categoria().rotulo()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaCategoria {
    UnidadeNaoObservada,
    /// Total ou faixa da categoria não publicáveis (`conferir_faixa_publicavel`).
    FaixaNaoPublicavel(#[from] crate::erro_web::ErroCaixa),
}
impl FalhaCategoria {
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::FaixaNaoPublicavel(_) => "a legenda da categoria não comprova total e faixa",
            Self::UnidadeNaoObservada => "a unidade não está identificada no Controle de Processos",
        }
    }
}

/// A lista da categoria na forma única, com a página observada.
pub(crate) fn selecionar(
    leitura: &Leitura,
    categoria: CategoriaLista,
    origem: &str,
) -> Result<Value, FalhaCategoria> {
    if leitura.unidade.is_none() {
        return Err(FalhaCategoria::UnidadeNaoObservada);
    }
    let (tabela, cobertura) = match categoria {
        CategoriaLista::Recebidos => (&leitura.caixa.recebidos, &leitura.recebidos),
        CategoriaLista::Gerados => (&leitura.caixa.gerados, &leitura.gerados),
    };
    paginas::conferir_faixa_publicavel(categoria.categoria(), tabela, cobertura)?;
    Ok(crate::saida::lista_da_categoria(
        leitura,
        categoria.categoria(),
        origem,
    ))
}

/// Publica ou exporta a lista da categoria a partir do HTML obtido,
/// com a conferência de `--unidade` da leitura da caixa.
pub(crate) fn publicar_lista(
    html: &str,
    unidade: Option<&str>,
    categoria: CategoriaLista,
    json_mode: bool,
    destino: Option<&Path>,
    nota: Option<&NotaDoRelogin>,
) -> ExitCode {
    let Ok(leitura) = paginas::juntar(&[html.to_owned()], unidade) else {
        return erro_fonte(json_mode, "Controle de Processos não reconhecido");
    };
    let mut modelo = match selecionar(&leitura, categoria, "web") {
        Ok(modelo) => modelo,
        Err(erro) => return erro_fonte(json_mode, erro.mensagem()),
    };
    if let Some(caminho) = destino {
        return match arquivo_caixa::gravar(caminho, &modelo) {
            Ok(bytes) => {
                if json_mode {
                    // O arquivo guarda só a lista; `relogin` vai em stdout.
                    let mut valor = json!({
                        "arquivo":"gravado",
                        "bytes":bytes,
                        "escopo":"categoria_verificada",
                        "categoria":categoria.rotulo()
                    });
                    relogin_cli::anotar_com_total(&mut valor, nota);
                    println!("{valor}");
                } else {
                    relogin_cli::avisar(json_mode, nota);
                    println!(
                        "IES: categoria verificada gravada em arquivo privado ({bytes} bytes)."
                    );
                }
                ExitCode::SUCCESS
            }
            Err(erro) => erro_fonte(json_mode, erro.mensagem()),
        };
    }
    if json_mode {
        relogin_cli::anotar_com_total(&mut modelo, nota);
        if let Err(erro) = crate::saida::imprimir_json(&modelo) {
            eprintln!("IES: falha ao emitir JSON: {erro}");
            return std::process::ExitCode::from(1);
        }
    } else {
        relogin_cli::avisar(json_mode, nota);
        let quantidade = modelo["processos"].as_array().map_or(0, Vec::len);
        println!(
            "IES: {}: {quantidade} processo(s) observado(s); total declarado: {}; faixa observada: {}.",
            categoria.rotulo(),
            modelo["total_declarado"],
            match &modelo["faixa_observada"] {
                Value::Null => "nenhuma".to_owned(),
                faixa => format!("{} a {}", faixa["inicio"], faixa["fim"]),
            }
        );
    }
    ExitCode::SUCCESS
}

/// Um GET; com `--pagina N`, no máximo um POST do paginador da categoria;
/// sessão na visão D: mais a troca para R, sem volta.
pub(crate) fn listar_categoria(
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
    categoria: CategoriaLista,
    pagina: Option<usize>,
) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.mensagem()),
    };
    // Cada tentativa relê a sessão e colhe na geração gravada agora.
    let lido = relogin_cli::ler(&arquivos, unidade, || {
        pagina_web::obter(&arquivos, unidade, categoria.categoria(), pagina)
    });
    match lido {
        Ok((html, nota)) => {
            publicar_lista(&html, unidade, categoria, json_mode, destino, nota.as_ref())
        }
        Err(falha) => relogin_cli::publicar_falha(json_mode, &falha),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginas;

    fn leitura(recebidos_total: usize) -> Leitura {
        let html = format!(
            "<a id='lnkInfraUnidade'>PPI</a>\
             <table id='tblProcessosRecebidos'>\
             <caption>{recebidos_total} registros - 1 a 1):</caption>\
             <tr id='P1'><td><a class='processoVisualizado'>R-0001</a></td></tr></table>\
             <table id='tblProcessosGerados'><caption>1 registros - 1 a 1):</caption>\
             <tr id='P2'><td><a class='processoVisualizado'>G-0002</a></td></tr></table>"
        );
        paginas::juntar(&[html], None).unwrap()
    }

    #[test]
    fn categoria_nao_publica_processos_sem_unidade_exibida() {
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = paginas::juntar(&[html.to_owned()], None).unwrap();
        assert_eq!(leitura.unidade, None);
        assert_eq!(
            selecionar(&leitura, CategoriaLista::Gerados, "web"),
            Err(FalhaCategoria::UnidadeNaoObservada)
        );
    }

    #[test]
    fn categoria_coberta_preserva_unidade_observada() {
        let modelo = selecionar(&leitura(1), CategoriaLista::Gerados, "web").unwrap();
        assert_eq!(modelo["unidade"], "PPI");
        assert_eq!(modelo["categoria"], "gerados");
    }

    fn aceita(args: &[&str]) -> bool {
        use clap::Parser;
        let mut todos = vec!["ies", "unidade"];
        todos.extend_from_slice(args);
        crate::Cli::try_parse_from(todos).is_ok()
    }

    #[test]
    fn listas_sao_subcomandos_e_listar_processos_pagina_a_visao_detalhada() {
        assert!(aceita(&["recebidos"]));
        assert!(aceita(&["gerados"]));
        assert!(aceita(&["gerados", "--saida", "lista.json"]));
        assert!(aceita(&["colunas"]));
        assert!(!aceita(&["listar-processos", "--pagina-arquivo", "p.html"]));
        assert!(!aceita(&["listar-processos", "--categoria", "gerados"]));
        assert!(aceita(&["listar-processos", "--pagina", "2"]));
        assert!(!aceita(&["gerados", "--pagina-arquivo", "pagina-2.html"]));
        assert!(!aceita(&["gerados", "--categoria", "gerados"]));
    }

    #[test]
    fn gerados_inteiros_sao_consultaveis_mesmo_se_recebidos_paginam() {
        let modelo = selecionar(&leitura(200), CategoriaLista::Gerados, "web").unwrap();
        assert_eq!(modelo["categoria"], "gerados");
        assert_eq!(modelo["total_declarado"], 1);
        assert_eq!(modelo["processos"][0]["numero"], "G-0002");
        assert!(modelo.get("recebidos").is_none());
        assert!(modelo.get("completa").is_none());
    }

    #[test]
    fn categoria_paginada_publica_a_pagina_com_total_e_faixa() {
        let modelo = selecionar(&leitura(200), CategoriaLista::Recebidos, "web").unwrap();
        assert_eq!(modelo["total_declarado"], 200);
        assert_eq!(modelo["faixa_observada"]["inicio"], 1);
        assert_eq!(modelo["faixa_observada"]["fim"], 1);
        assert_eq!(modelo["processos"][0]["numero"], "R-0001");
        assert!(modelo.get("completa").is_none());
    }

    #[test]
    fn categoria_com_legenda_sem_faixa_e_total_maior_e_recusada() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
             <table id='tblProcessosRecebidos'><caption>5 registros:</caption>\
             <tr id='P1'><td><a class='processoVisualizado'>R-0001</a></td></tr></table>\
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = paginas::juntar(&[html.to_owned()], None).unwrap();
        let _erro = selecionar(&leitura, CategoriaLista::Recebidos, "web").unwrap_err();
        assert!(selecionar(&leitura, CategoriaLista::Gerados, "web").is_ok());
    }

    #[test]
    fn pagina_das_listas_exige_inteiro_positivo() {
        for lista in ["recebidos", "gerados"] {
            assert!(aceita(&[lista, "--pagina", "2"]));
            assert!(!aceita(&[lista, "--pagina", "0"]));
            assert!(!aceita(&[lista, "--pagina", "-1"]));
            assert!(!aceita(&[lista, "--pagina", "x"]));
        }
    }

    #[test]
    fn lista_tem_a_forma_unica_da_caixa() {
        let modelo = selecionar(&leitura(200), CategoriaLista::Recebidos, "web").unwrap();
        let chaves: Vec<&str> = modelo
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            chaves,
            [
                "categoria",
                "faixa_observada",
                "origem",
                "processos",
                "total_declarado",
                "unidade"
            ]
        );
        assert_eq!(
            modelo,
            crate::saida::lista_da_categoria(
                &leitura(200),
                crate::dominio::Categoria::Recebidos,
                "web"
            )
        );
    }

    fn exportar_lista(
        unidade: Option<&str>,
        categoria: CategoriaLista,
    ) -> (ExitCode, Option<Value>) {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
             <table id='tblProcessosRecebidos'><caption>(3 registros - 1 a 1):</caption>\
             <tr id='P1'><td><a class='processoVisualizado'>R-0001</a></td></tr></table>\
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let caminho = std::env::temp_dir().join(format!(
            "ies-lista-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let codigo = publicar_lista(html, unidade, categoria, true, Some(&caminho), None);
        let valor = std::fs::read(&caminho)
            .ok()
            .map(|bytes| serde_json::from_slice(&bytes).unwrap());
        let _ = std::fs::remove_file(&caminho);
        (codigo, valor)
    }

    #[test]
    fn lista_confere_unidade_global_antes_de_publicar() {
        let (codigo, valor) = exportar_lista(Some("OUTRA"), CategoriaLista::Recebidos);
        assert_eq!(codigo, ExitCode::from(1));
        assert!(valor.is_none());
        let (codigo, valor) = exportar_lista(Some("PPI"), CategoriaLista::Gerados);
        assert_eq!(codigo, ExitCode::SUCCESS);
        let valor = valor.unwrap();
        assert_eq!(valor["categoria"], "gerados");
        assert_eq!(valor["total_declarado"], 0);
        assert!(valor.as_object().unwrap().get("recebidos").is_none());
    }

    #[test]
    fn lista_exportada_e_um_objeto_e_nao_um_array() {
        let (codigo, valor) = exportar_lista(None, CategoriaLista::Recebidos);
        assert_eq!(codigo, ExitCode::SUCCESS);
        let valor = valor.unwrap();
        assert!(valor.is_object());
        assert_eq!(valor["categoria"], "recebidos");
        assert_eq!(valor["faixa_observada"]["fim"], 1);
        assert_eq!(valor["processos"][0]["numero"], "R-0001");
    }

    #[test]
    fn categoria_unica_com_todas_posicoes_preserva_numeros() {
        let modelo = selecionar(&leitura(1), CategoriaLista::Recebidos, "web").unwrap();
        assert_eq!(modelo["processos"][0]["numero"], "R-0001");
        assert_eq!(modelo["total_declarado"], 1);
        assert!(modelo.get("gerados").is_none());
    }
}
