//! As ações do SEI que o IES lê, confere ou envia (#731): `acao=` do
//! `controlador.php` e `acao_ajax=` do `controlador_ajax.php`. O valor na URL
//! é do SEI; aqui cada parâmetro é um conjunto fechado, para que cada
//! comparação e cada `match` sobre a ação passe pelo compilador. Uma ação
//! fora do conjunto não é lida: `ler` devolve `None`, que é "não é a ação
//! procurada".

macro_rules! acoes_do_sei {
    ($(#[$doc:meta])* $nome:ident { $($variante:ident => $na_url:literal,)* }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub(crate) enum $nome {
            $($variante,)*
        }

        impl $nome {
            /// O valor na URL como o SEI o publica.
            pub(crate) const fn como_na_url(self) -> &'static str {
                match self {
                    $(Self::$variante => $na_url,)*
                }
            }

            /// A ação cujo valor na URL é exatamente `valor`.
            pub(crate) fn ler(valor: &str) -> Option<Self> {
                match valor {
                    $($na_url => Some(Self::$variante),)*
                    _ => None,
                }
            }
        }

        /// Na saída JSON, a ação aparece como o SEI a publica.
        impl serde::Serialize for $nome {
            fn serialize<S: serde::Serializer>(&self, serializador: S) -> Result<S::Ok, S::Error> {
                serializador.serialize_str(self.como_na_url())
            }
        }
    };
}

acoes_do_sei! {
    /// Uma ação do `controlador.php`, com o valor exato de `acao=`.
    AcaoDoSei {
        AcompanhamentoCadastrar => "acompanhamento_cadastrar",
        AcompanhamentoExcluir => "acompanhamento_excluir",
        AcompanhamentoGerenciar => "acompanhamento_gerenciar",
        AndamentoMarcadorCadastrar => "andamento_marcador_cadastrar",
        AndamentoMarcadorGerenciar => "andamento_marcador_gerenciar",
        AndamentoMarcadorRemover => "andamento_marcador_remover",
        AnotacaoRegistrar => "anotacao_registrar",
        ArvoreVisualizar => "arvore_visualizar",
        BlocoAssinaturaListar => "bloco_assinatura_listar",
        BlocoEscolher => "bloco_escolher",
        BlocoInternoListar => "bloco_interno_listar",
        BlocoReuniaoListar => "bloco_reuniao_listar",
        ComentarioCadastrar => "comentario_cadastrar",
        ComentarioListar => "comentario_listar",
        ControlePrazoDefinir => "controle_prazo_definir",
        ControlePrazoExcluir => "controle_prazo_excluir",
        ControlePrazoListar => "controle_prazo_listar",
        DocumentoAlterar => "documento_alterar",
        DocumentoAlterarRecebido => "documento_alterar_recebido",
        DocumentoAssinar => "documento_assinar",
        DocumentoCancelar => "documento_cancelar",
        DocumentoCiencia => "documento_ciencia",
        DocumentoConsultar => "documento_consultar",
        DocumentoConsultarRecebido => "documento_consultar_recebido",
        DocumentoDownloadAnexo => "documento_download_anexo",
        DocumentoEscolherTipo => "documento_escolher_tipo",
        DocumentoGerar => "documento_gerar",
        DocumentoReceber => "documento_receber",
        DocumentoVersaoListar => "documento_versao_listar",
        DocumentoVisualizar => "documento_visualizar",
        EditorSalvar => "editor_salvar",
        GrupoUnidadeCadastrar => "grupo_unidade_cadastrar",
        GrupoUnidadeListar => "grupo_unidade_listar",
        InfraTrocarUnidade => "infra_trocar_unidade",
        Login => "login",
        Principal => "principal",
        ProcedimentoAlterar => "procedimento_alterar",
        ProcedimentoAnexar => "procedimento_anexar",
        ProcedimentoAtribuicaoCadastrar => "procedimento_atribuicao_cadastrar",
        ProcedimentoAtualizarAndamento => "procedimento_atualizar_andamento",
        ProcedimentoCiencia => "procedimento_ciencia",
        ProcedimentoConcluir => "procedimento_concluir",
        ProcedimentoConfigurarDetalhe => "procedimento_configurar_detalhe",
        ProcedimentoConsultar => "procedimento_consultar",
        ProcedimentoConsultarHistorico => "procedimento_consultar_historico",
        ProcedimentoControlar => "procedimento_controlar",
        ProcedimentoEnviar => "procedimento_enviar",
        ProcedimentoEscolherTipo => "procedimento_escolher_tipo",
        ProcedimentoExcluirRelacionamento => "procedimento_excluir_relacionamento",
        ProcedimentoGerar => "procedimento_gerar",
        ProcedimentoReabrir => "procedimento_reabrir",
        ProcedimentoRelacionar => "procedimento_relacionar",
        ProcedimentoRemoverSobrestamento => "procedimento_remover_sobrestamento",
        ProcedimentoSobrestar => "procedimento_sobrestar",
        ProcedimentoTrabalhar => "procedimento_trabalhar",
        ProcedimentoVisualizar => "procedimento_visualizar",
        ProtocoloCienciaListar => "protocolo_ciencia_listar",
        ProtocoloPesquisaRapida => "protocolo_pesquisa_rapida",
        ProtocoloPesquisar => "protocolo_pesquisar",
        RelBlocoProtocoloListar => "rel_bloco_protocolo_listar",
        UnidadeSelecionarEnvioProcesso => "unidade_selecionar_envio_processo",
        UsuarioValidarAcesso => "usuario_validar_acesso",
    }
}

acoes_do_sei! {
    /// Uma ação do `controlador_ajax.php`, com o valor exato de `acao_ajax=`.
    AcaoAjaxDoSei {
        ContatoAutoCompletarContextoRi1225 => "contato_auto_completar_contexto_RI1225",
        HipoteseLegalSelectNomeBaseLegal => "hipotese_legal_select_nome_base_legal",
        ProcedimentoAnexacaoVerificar => "procedimento_anexacao_verificar",
        ProtocoloRi1023 => "protocolo_RI1023",
        UnidadeAutoCompletarEnvioProcesso => "unidade_auto_completar_envio_processo",
    }
}

/// A ação que um POST derivado envia: do `controlador.php` ou do
/// `controlador_ajax.php`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AcaoDaSubmissao {
    Controlador(AcaoDoSei),
    Ajax(AcaoAjaxDoSei),
}
