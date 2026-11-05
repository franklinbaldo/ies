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

Este contrato especifica o comportamento alvo, não declara a
capacidade disponível na CLI. Estado `partial`: existe apenas o núcleo de seleção e reconciliação offline, sem o comando de rede. Nenhuma escrita está disponível.

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
Replay, RED, gate, live de prévia: **NOT_RUN** nesta etapa.
Escrita live: **NOT_RUN até "escrita autorizada" na issue #246**.
