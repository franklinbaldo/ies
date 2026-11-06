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

Estado `partial`: a CLI oferece prévia nas telas com link direto e assinado para o cadastro (`documento_gerar`) **e** nas telas com `escolher(ID)` quando a função servida comprova duas instruções literais: atribuir `hdnIdSerie` e submeter `frmDocumentoEscolherTipo`. Nessa segunda forma, faz um POST **de navegação sem criação** para `documento_escolher_tipo`, valida a Location 302 assinada do mesmo processo e faz um GET do cadastro; não executa JS, não segue redirecionamentos automaticamente e não repete requisições. A confirmação tem um único POST **de escrita** e reconciliação por releitura, somente com `--nivel publico` explícito e script observado. Até CI, replay e ensaio live, tudo permanece WIP, sem validação para uso real. Níveis restrito/sigiloso recusam antes de escrever, pois dependem de hipótese legal não resolvida.

## Prévia

Abrir o processo, confirmar identidade e unidade da sessão, constatar
que o processo está aberto nessa unidade e seguir o link **publicado**
"Incluir Documento" (`documento_escolher_tipo`). Ler a lista
`tblSeries`, recusar nomes ausentes e ambíguos, ignorar "Externo",
abrir o cadastro do tipo exato (GET direto ou POST de navegação seguido de GET) e conferir `lblSerieTitulo`, formulário,
campos e nível solicitado. A prévia não envia POST de criação. Uma
eventual leitura por POST (`hdnFiltroSerie=T`) exige declaração de
efeito separada e contagem; `documento_gerar_multiplo` não é leitura.

## Confirmação

Antes do primeiro POST de escrita, todas as recusas precisam estar
resolvidas. Derivar exatamente um formulário `documento_gerar` do
HTML servido pelo primitivo `formulario_web`, com `EfeitoAvaliado`.
Não construir URL assinada, executar JS, enviar campos fora da lista
observada, repetir POST nem fazer fallback.
Quando o cadastro exigir controles HTML `required`, recusar a escrita antes do POST se algum controle ativo não puder ser atendido pela CLI, inclusive descrição vazia. Controles `hidden` e desabilitados não exigem escolha do usuário. Antes da tentativa, recusar sem POST uma árvore inicial incompleta, um processo sem ID ou IDs de documentos repetidos ou vazios. Depois da tentativa,
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
| `link_do_tipo_de_documento_nao_navegavel` | O onclick/script não comprova exatamente a navegação segura por formulário |
| `endereco_do_tipo_de_documento_nao_reconhecido` | URL do tipo ausente, inválida ou vinculada a outro processo |
| `tipo_do_cadastro_diferente_do_solicitado` | A tela aberta não identifica o tipo pedido |
| `descricao_nao_oferecida_no_cadastro` | A descrição solicitada não tem controle utilizável |
| `nivel_nao_oferecido_no_cadastro` | O nível solicitado não aparece habilitado |
| `campo_obrigatorio_do_cadastro_nao_atendido` | O cadastro publica controle HTML obrigatório que a CLI não consegue preencher com valor observado |
| `tela_de_criacao_redirecionada` | A tela de leitura devolveu 302 não seguido |
| `nivel_de_documento_nao_informado_para_escrita` | Confirmação exige --nivel publico explícito; não presumir nível |
| `nivel_de_documento_requer_hipotese_legal` | Nível não público exige resolver a hipótese legal antes da escrita |
| `submissao_do_documento_nao_observada` | Cadastro não contém a regra observada de submissão |
| `arvore_de_documentos_incompleta` | Pasta fechada impede comparação de documentos |
| `foto_inicial_de_documentos_inconsistente` | Antes do POST, a árvore contém IDs de documento repetidos/vazios ou ID de processo ausente; nenhuma escrita é enviada |
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
O replay GO das duas cassettes é **obrigatório no CI**: ZIP e membros conferidos por hash; 15 respostas-alvo (3 lidas, 5 recusadas e 7 não aplicáveis). [CI 42127145672](https://github.com/franklinbaldo/ies/actions/runs/42127145672): PASS em prek, Release e replay (head `6603a36`).

Os testes sintéticos verificam prévia de 5 GET/0 POST e confirmação de 6 GET/1 POST, com releitura e efeito incerto sem retry. [RED 42120924295](https://github.com/franklinbaldo/ies/actions/runs/42120924295): 1 teste falhou antes da guarda de nível pré-rede. Um teste de 302 sem Location apresentou [RED 42131591459](https://github.com/franklinbaldo/ies/actions/runs/42131591459) (1 falha, 1.610 aprovados); a correção exige Location não vazia para reconhecer o 302 como resposta conferível. O head corrigido requer CI próprio. RED e compilação Rust **locais: NOT_RUN** (cargo/rustc e DNS indisponíveis).

**Live:** login, prévia, escrita e releitura em treinamento **NOT_RUN**, 0 requisições, 0 POSTs e nenhum resíduo nesta sessão. A escrita está autorizada só em treinamento; falta a sequência prévia observada → uma única confirmação → leitura independente, com número SEI e resíduo registrados.

## Guarda de script e causas da retomada 🐙

A confirmação não pode inferir a instrução de submissão de uma
substring arbitrária do HTML. Exige uma única atribuição literal
observada de `hdnFlagDocumentoCadastro` em `script` ativo;
`template`, `script type=application/json`, comentário de bloco
e script duplicado não autorizam a escrita. Os quatro casos são
testados com transporte injetável e exatamente 5 GET / 0 POST.
Formas não reconhecidas recusam com
`submissao_do_documento_nao_observada` antes do POST.

As causas da prévia, escolha e barra de inclusão são compostas por
`#[derive(Causa)]`, sem mapeamento manual de subcódigos. A saída
legada mantém as folhas de erro publicadas, com caminho tipado
interno e preservação da mensagem/status HTTP quando observados.
A escrita incerta continua saída 3.


A guarda de confirmação também recusa `script[src]`, cujo corpo inline
não é executado pelo navegador, e atribuições presentes apenas em
literais JavaScript multilinha. Esses casos têm testes sintéticos com
**5 GET, 0 POST criador**: formas inertes não autorizam escrita.

## Fronteira de Location da confirmação

O retorno 302 somente pode contribuir para uma confirmação se a
`Location` publicada permanecer no controlador HTTPS da mesma instalação,
com uma ação explícita e única. Um 302 sem Location, com host externo,
com caminho alheio ou sem ação não comprova atribuição causal da mudança
na árvore: efeito incerto (saída 3), mesmo quando se observa um novo
documento. A leitura de reconciliação ocorre uma vez após o POST;
não seguir Location e não repetir a escrita.
