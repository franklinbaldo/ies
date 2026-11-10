//! #332 — conecta a confirmação única do PDF à saída pública.
//! Validada ao vivo na `UnB` de treinamento (09/11/2026, saída 0).
use crate::Cli;
use crate::documento_externo_incluir::{
    PdfLocal,
    fluxo_confirmar::{ErroConfirmacaoExterna, executar_confirmacao_https},
};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use serde_json::{Value, json};
use std::path::Path;
use std::process::ExitCode;

/// Os contadores incluem as tentativas de rede que falharam. O multipart
/// usa o mesmo cliente HTTPS, mas não passa pelo Contador dos GET/POST comuns.
pub(super) fn executar(
    cli: &Cli,
    processo: &NumeroProcesso,
    tipo: &str,
    pdf: &PdfLocal,
    caminho_pdf: &Path,
) -> ExitCode {
    com_sessao(cli, |sessao, indice| {
        // Aprovada em treinamento (09/11/2026): grava também em produção.
        if let Err(causa) = crate::origem_de_treinamento::exigir(
            crate::origem_de_treinamento::Escrita::DocumentoIncluirExterno,
            sessao.origem,
        ) {
            return crate::origem_de_treinamento::recusa(cli.json, causa);
        }
        let mut envio = EnvioHttps;
        let mut contagem = Contador::novo(&mut envio);
        let mut upload_tentado = false;
        let resultado = executar_confirmacao_https(
            sessao,
            processo,
            cli.unidade.as_deref(),
            tipo,
            pdf,
            caminho_pdf,
            indice,
            &mut contagem,
            &mut upload_tentado,
        );
        let requisicoes = contagem.feitas().saturating_add(u8::from(upload_tentado));
        match resultado {
            Ok(feito) => {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "escopo": "confirmacao_web",
                            "processo": processo.as_str(),
                            "tipo_solicitado": tipo,
                            "numero_sei": feito.numero_sei,
                            "unidade": feito.unidade,
                            "confirmado": true,
                            "escrita_remota": true,
                            "upload_tentado": true,
                            "cadastro_tentado": true,
                            "reconciliado_por_leitura": true,
                            "requisicoes": requisicoes
                        })
                    );
                } else {
                    println!(
                        "Documento externo {} incluído no processo {} (unidade {}), comprovado por releitura independente. Upload temporário: 1 tentativa; cadastro definitivo: 1 tentativa. Requisições: {}.",
                        feito.numero_sei,
                        processo.as_str(),
                        feito.unidade,
                        requisicoes
                    );
                }
                ExitCode::SUCCESS
            }
            Err(erro) => {
                if cli.json {
                    println!("{}", json_da_falha(&erro, upload_tentado, requisicoes));
                } else {
                    eprintln!(
                        "IES: inclusão externa não comprovada na etapa {} (saída {}). Upload tentado: {}; cadastro tentado: {}; resíduo temporário possível: {}; requisições: {}. Não repita automaticamente a escrita.",
                        erro.etapa(),
                        erro.codigo_saida(),
                        upload_tentado,
                        erro.cadastro_tentado(),
                        erro.temporario_possivel(),
                        requisicoes
                    );
                }
                ExitCode::from(erro.codigo_saida())
            }
        }
    })
}

fn json_da_falha(erro: &ErroConfirmacaoExterna, upload_tentado: bool, requisicoes: u8) -> Value {
    json!({
        "confirmado": false,
        "etapa": erro.etapa(),
        "efeito_incerto": erro.codigo_saida() == 3,
        "retentavel": false,
        "upload_tentado": upload_tentado,
        "cadastro_tentado": erro.cadastro_tentado(),
        "temporario_possivel": erro.temporario_possivel(),
        "requisicoes": requisicoes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documento_externo_incluir::confirmacao::FalhaConfirmacao;

    #[test]
    fn bdd_332_04_efeito_incerto_informa_etapa_e_residuo_sem_repeticao() {
        let erro = ErroConfirmacaoExterna::Sequencia(FalhaConfirmacao::EfeitoIncerto);
        let resultado = json_da_falha(&erro, true, 7);
        assert_eq!(resultado["confirmado"], false);
        assert_eq!(resultado["etapa"], "releitura_independente");
        assert_eq!(resultado["requisicoes"], 7);
        assert_eq!(resultado["upload_tentado"], true);
        assert_eq!(resultado["cadastro_tentado"], true);
        assert_eq!(resultado["temporario_possivel"], true);
        assert_eq!(resultado["retentavel"], false);
    }

    #[test]
    fn foto_inicial_incompleta_nao_finge_upload_tentado() {
        let erro = ErroConfirmacaoExterna::Sequencia(FalhaConfirmacao::FotoInicialIncompleta);
        let resultado = json_da_falha(&erro, false, 4);
        assert_eq!(resultado["etapa"], "fotografia_inicial");
        assert_eq!(resultado["upload_tentado"], false);
        assert_eq!(resultado["cadastro_tentado"], false);
        assert_eq!(resultado["temporario_possivel"], false);
    }
}
