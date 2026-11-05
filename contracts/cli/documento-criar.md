---
type: CommandContract
title: Criar documento interno no processo pelo cadastro Web
operation: documento.criar
command: ies documento criar --processo NUP --tipo TIPO [--descricao TEXTO] [--nivel NIVEL] [--confirmar]
effect: write
implementation: partial
source_commit: 323eb01
---

# Criar documento interno — issue #246

## Limite

Operação planejada: `ies documento criar --processo NUP --tipo TIPO
[--descricao TEXTO] [--nivel publico|restrito|sigiloso] [--confirmar]`.
O valor `--nivel` e eventuais hipóteses legais devem ser conferidos
contra o cadastro servido; não há autorização para supor uma seleção
implícita. A operação **não** grava texto no editor e **não** assina.

Estado `partial`: a CLI oferece prévia de leitura nas telas com link direto e assinado para o cadastro (`documento_gerar`). A confirmação agora tem um caminho de POST único e reconciliação por releitura, **somente para o nível público e formulário com o script observado**; antes de receber CI e replay, essa implementação permanece WIP, não validada para uso real. Instalações que só publicam `escolher(ID)` recusam sem inventar navegação. Níveis restrito/sigiloso recusam antes de escrever, pois dependem de hipótese legal não resolvida.

## Prévia

Abrir o processo, confirmar identidade e unidade da sessão, constatar
que o processo está aberto nessa unidade e seguir o link **publicado**
"Incluir Documento" (`documento_escolher_tipo`). Ler a lista
`tblSeries`, recusar nomes ausentes e ambíguos, ignorar "Externo",
abrir o cadastro do tipo exato e conferir `lblSerieTitulo`, formulário,
campos e nível solicitado. A prévia não envia POST de criação. Uma
eventual leitura por POST (`hdnFiltroSerie=T`) exige declaração de
efeito separada e contagem; `documento_gerar_multiplo` não é leitura.

## Confirmação

Antes do primeiro POST de escrita, todas as recusas precisam estar
resolvidas. Derivar exatamente um formulário `documento_gerar` do
HTML servido pelo primitivo `formulario_web`, com `EfeitoAvaliado`.
Não construir URL assinada, executar JS, enviar campos fora da lista
observada, repetir POST nem fazer fallback. Depois da tentativa,
reabrir o processo e conferir a diferença entre as árvores.

Uma escrita comprovada exige **exatamente um novo documento** (id
único, número SEI publicado, tipo igual ao solicitado) e a preservação
dos documentos anteriores. O `id_documento` da resposta não substitui
o número SEI da árvore. Se o POST puder ter sido enviado e a verificação
falhar, ou se a mudança for ambígua, saída 3 (`efeito_incerto`).

## Recusas tipadas

| Subcódigo | Critério |
| --- | --- |
| `processo_nao_aberto_na_unidade_da_sessao` | A situação conferida não inclui a unidade ativa |
| `tipo_de_documento_nao_oferecido` | Não há opção exata na lista completa |
| `tipo_de_documento_ambiguo` | Mais de uma opção com o mesmo nome |
| `lista_de_tipos_fora_da_forma_observada` | `tblSeries` ou links não interpretáveis |
| `cadastro_de_documento_fora_da_forma_observada` | Cadastro ou tipo aberto inconsistente |
| `documento_criado_nao_comprovado` | Verificação concluiu que não surgiu documento |
| `efeito_incerto_na_criacao_de_documento` | Resposta ou árvore não permite conferir unicidade e identidade |
| `barra_do_incluir_documento_fora_da_forma` | A barra da raiz não tem estrutura interpretável |
| `barra_do_incluir_documento_repetida` | A barra da raiz aparece duas vezes |
| `links_de_incluir_documento_repetidos` | A barra publica mais de uma ação de incluir |
| `endereco_de_incluir_documento_nao_reconhecido` | O link de inclusão não é a URL relativa assinada esperada |
| `endereco_de_incluir_documento_de_outro_processo` | O link de inclusão aponta para outra identidade |
| `incluir_documento_nao_publicado_na_barra` | A unidade não publicou a opção de incluir |
| `unidade_da_sessao_nao_publicada` | A tela não identifica a unidade de trabalho |
| `situacao_do_processo_nao_lida_antes_de_criar_documento` | A árvore não publicou a situação segura para gravação |
| `link_do_tipo_de_documento_nao_navegavel` | A escolha depende de script ainda sem navegação derivada |
| `endereco_do_tipo_de_documento_nao_reconhecido` | URL do tipo ausente, inválida ou vinculada a outro processo |
| `tipo_do_cadastro_diferente_do_solicitado` | A tela aberta não identifica o tipo pedido |
| `descricao_nao_oferecida_no_cadastro` | A descrição solicitada não tem controle utilizável |
| `nivel_nao_oferecido_no_cadastro` | O nível solicitado não aparece habilitado |
| `tela_de_criacao_redirecionada` | A tela de leitura devolveu 302 não seguido |
| `nivel_de_documento_requer_hipotese_legal` | Nível não público exige resolver a hipótese legal antes da escrita |
| `submissao_do_documento_nao_observada` | Cadastro não contém a regra observada de submissão |
| `arvore_de_documentos_incompleta` | Pasta fechada impede comparação de documentos |
| `criacao_de_documento_ainda_nao_validada` | Subcódigo histórico da etapa anterior, não emitido pelo novo fluxo |
| `tipo_de_documento_vazio` | O tipo informado está vazio |
| `parametro_de_criacao_invalido` | Falha de validação de entrada |


Erros de abertura, atributos, transporte e formulário preservam seus
subcódigos próprios. Entrada inválida sai 2; recusa antes do POST sai 1;
sucesso demonstrado sai 0; efeito incerto após o POST sai 3.

## Saída projetada

Prévia: `{"nup":"…","tipo":"Despacho","confirmado":false,
"alterado":false,"requisicoes":N}`.
Após escrita comprovada: `{"nup":"…","tipo":"Despacho",
"confirmado":true,"alterado":true,"numero_sei":"…",
"requisicoes":N}`. Nenhum segredo ou URL assinada na saída.

## Evidências e validação

[Observação](../../docs/como-o-sei-funciona/inclusao-documento-interno.md)
e [decisão](../../docs/decisoes/documento-criar.md).
Oráculo: Todos `src/todos/documentos.py` (`criar`,
`_criar_na_arvore`, `cadastrar_da_tela`, `_depois_de_enviado`).
Cassettes Goiás `documento-incluir` e `documento-escritas`.
Replay, RED local, gate local e prévia live: **NOT_RUN** nesta etapa; os novos testes de formulário e de identidade foram publicados, mas não executados localmente por ausência de Rust e DNS. Contagem RED observada: **não aferida**. O CI do head precisa ser confirmado antes de considerar GREEN.
Escrita live: **AUTORIZADA exclusivamente em treinamento**, mas **NOT_RUN**. A autorização está na issue #246; executar apenas depois de validar o branch, fazer prévia ao vivo e comprovar a leitura posterior, registrando o resíduo. Nenhum documento foi criado nesta sessão.
