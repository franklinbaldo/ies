---
type: Guide
title: Catálogos e valores de formulários SEI
description: Modelagem de cargo de assinatura, classificação e opções administrativas
---

# Catálogos e valores de formulários

Complementa [opções de formulários](opcoes-formularios.md) e
[entidades candidatas](entidades-dos-formularios.md).
Não converter cada seletor em entidade, nem guardar opções como textos
livres quando existe um catálogo administrado por instalação.

## Cargo ou função na assinatura

O manual **Assinatura das Unidades** confirma que a administração
configura os cargos/funções disponíveis para cada unidade. A assinatura
de documento seleciona cargo/função dentre as opções autorizadas.

Separar três conceitos:
- **Cargo/função configurado**: opção cadastrada para assinatura na
  unidade, não necessariamente o cargo funcional da pessoa.
- **Escolha para o ato**: usuário/assinante seleciona opção permitida
  na unidade e contexto do documento.
- **Cargo/função registrado na assinatura**: informação histórica
  do ato, não deve mudar retroativamente se o catálogo mudar.

Candidato: `CargoFuncaoAssinatura` como referência de catálogo
escopada à instalação/órgão/unidade; `Assinatura` referencia a
opção escolhida e preserva o texto registrado, quando disponível.
Não inferir identidade por texto de cargo nem oferecer opções de outra
unidade. A configuração administrativa não prova autorização do
usuário para assinar.

## Tipo e classificação de processo

A expressão informal **classe de processo** precisa ser desambiguada
em termos documentados pelo SEI:

1. **Tipo de Processo**: categoria operacional configurada, com
   descrição, restrições de órgão/unidade e níveis de acesso
   permitidos/sugeridos.
2. **Classificação por Assuntos**: um ou mais códigos do plano de
   classificação documental, sugeridos pelo tipo de processo e
   alteráveis apenas conforme regras da gestão documental.
3. **Prioridade do processo**: opção administrativa própria do
   formulário de inclusão a partir do SEI 4.1; não é a classe/tipo
   nem o assunto.
4. **Hipótese Legal**: referência da restrição de acesso, distinta
   do assunto e do tipo.

Proposta: `TipoProcesso` e `AssuntoClassificacao` como catálogos
referenciáveis; vínculo `TipoProcessoAssunto` com sugestão
configurada; `PrioridadeProcesso` como valor/registro conforme a
interface; políticas de acesso por contexto. Não inventar `Classe`
genérica antes de comprovar que a instalação usa esse identificador.

## Outros catálogos relevantes

| Formulário | Conceito | Decisão |
| --- | --- | --- |
| Criar documento | Tipo de Documento | Referência administrada, não texto solto |
| Criar documento interno | Modelo de Documento / Texto Padrão | Fonte de conteúdo escolhida; objetos diferentes |
| Tipo de Documento | Veículo de Publicação | Referência configurável e permissão de publicação |
| Tipo de Documento | Permite Interessados/Destinatários | Políticas do tipo, não atributos de cada documento |
| Tipo de Processo | Exclusivo da ouvidoria | Política por perfil/unidade |
| Tipo de Processo | Processo único por interessado | Regra de validação no escopo configurado |
| Cadastro de contato | Tipo/Categoria de Contato | Catálogos administráveis distintos da pessoa |
| Cadastro de contato | Vocativo/Título/Tratamento | Valores de apresentação, não identidade |
| Acesso externo | Acesso integral/parcial e validade | Escopo e duração da concessão |
| Acesso externo para assinatura | Permissão de incluir documentos e tipos permitidos | Escopo autorizativo, não atributo do usuário |

## Regra de modelagem

- **Entidade**: vida própria, ID, relacionamento ou gerenciamento
  persistente, como `TipoProcesso`, `TipoDocumento`,
  `CargoFuncaoAssinatura`, `ModeloDocumento`.
- **Relação**: política/restrição ou escolha entre entidades,
  como tipo de processo e assuntos sugeridos.
- **Valor**: enum ou referência controlada, como nível de acesso
  e prioridade.
- **Parâmetro de operação**: escolha que altera efeito do comando,
  como manter aberto após envio.
- **Apresentação**: label de campo, vocativo e formatação; não
  converter em identidade técnica.

Todas as referências dependem de **instalação e contexto**. Mesmo
nome não implica mesmo ID entre instalações. Um ID não deve ser
suposto estável por versão. Ausência no formulário pode significar
restrição de permissão, não inexistência do catálogo.

## Testes discriminantes

1. A mesma pessoa assina em unidades A e B com cargos disponíveis
   diferentes; a assinatura preserva o cargo escolhido na época.
2. Mudar a configuração de cargo não altera assinaturas passadas.
3. Um Tipo de Processo sugere mais de um Assunto; não reduzir a
   classificação a uma string do próprio tipo.
4. Um tipo está visível numa unidade e oculto em outra; não tomar
   lista de opções como catálogo global da instalação.
5. Um tipo sugere nível restrito, mas permite outros; sugestão
   não é restrição obrigatória.
6. Prioridade, nível de acesso e assunto são independentes:
   não criar enum que misture suas categorias.
7. Documento de tipo que não permite destinatário deve rejeitar
   o campo, conforme política comprovada da instalação.

## Fontes oficiais

- [Assinaturas das Unidades](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administracao_SEI-01_Assinatura_das_Unidades.html)
- [Tipo de Processo](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administra%C3%A7%C3%A3o_SEI-05_Tipo_de_Processo.html)
- [Operações básicas com processos](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Operacoes_basicas_com_processos.html)
- [Tipos de Documento](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administra%C3%A7%C3%A3o_SEI-07_Tipo_de_Documento.html)
- [Operações básicas com documentos](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Operacoes_basicas_com_documentos.html)
- [Usuário externo](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Usuario_externo.html)

Decisões conceituais fundamentadas nos manuais. Campos e IDs concretos
devem ser conferidos nos formulários capturados antes de criar structs
Rust. Não altera runtime, JSON ou autorização de transportes.
